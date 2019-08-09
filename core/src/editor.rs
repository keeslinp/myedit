use crossbeam_channel::{unbounded, Sender};
use libloading::os::unix::Symbol;
use notify::{watcher, DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::any::Any;
use std::collections::HashMap;
use std::default::Default;
use std::ffi::c_void;
use std::{fs, path, time};
use termion::raw::IntoRawMode;
use types::{BackBuffer, Cmd, Cursor, GlobalData, Mode, Msg, Point, Utils};

use crate::back_buffer;
use crate::utils;

#[derive(Debug)]
struct DynLib {
    lib: libloading::Library,
    render_fn: Symbol<extern "C" fn(&GlobalData, &mut BackBuffer, &Utils, *mut c_void)>,
    update_fn: Symbol<extern "C" fn(&mut GlobalData, &Msg, &Utils, &Box<Fn(Cmd)>, *mut c_void)>,
    cleanup_fn: Symbol<extern "C" fn(*mut c_void)>,
    data: *mut c_void,
}

impl Drop for DynLib {
    fn drop(&mut self) {
        (self.cleanup_fn)(self.data);
    }
}

fn load_lib(path: &path::PathBuf) -> DynLib {
    let file_name = path.file_name().expect("getting lib name");
    let copy_path: path::PathBuf = [
        "./lib_copies",
        file_name.to_str().expect("converting path string"),
    ]
    .iter()
    .collect();
    fs::copy(path, copy_path.clone()).expect("copying lib");
    unsafe {
        let lib = libloading::Library::new(copy_path).expect("loading lib");
        let render_fn: libloading::Symbol<
            extern "C" fn(&GlobalData, &mut BackBuffer, &Utils, *mut c_void),
        > = lib.get(b"render").expect("loading render function");
        let update_fn: libloading::Symbol<
            extern "C" fn(&mut GlobalData, &Msg, &Utils, &Box<Fn(Cmd)>, *mut c_void),
        > = lib.get(b"update").expect("loading update function");
        let init_fn: libloading::Symbol<extern "C" fn() -> *mut c_void> =
            lib.get(b"init").expect("loading init function");
        let cleanup_fn: libloading::Symbol<extern "C" fn(*mut c_void)> =
            lib.get(b"cleanup").expect("loading cleanup function");
        let data = init_fn();
        DynLib {
            render_fn: render_fn.into_raw(),
            update_fn: update_fn.into_raw(),
            cleanup_fn: cleanup_fn.into_raw(),
            data,
            lib,
        }
    }
}

// #[cfg(debug_assertion)]
const LIB_LOC: &'static str = "./target/debug";

// #[cfg(not(debug_assertion))]
// const LIB_LOC: &'static str = "./target/release";

fn load_libs(watcher: &mut RecommendedWatcher) -> HashMap<String, DynLib> {
    use std::fs::read_dir;
    read_dir(LIB_LOC)
        .expect("reading lib folder")
        .map(|dir_buff| dir_buff.unwrap().path())
        .filter(|path| path.extension().map(|ext| ext == "dylib").unwrap_or(false))
        .inspect(|path| watcher.watch(path, RecursiveMode::NonRecursive).unwrap())
        .map(|path| {
            (
                path.file_name().unwrap().to_str().unwrap().to_owned(),
                load_lib(&path),
            )
        })
        .collect()
}

fn initial_state() -> GlobalData {
    use generational_arena::Arena;
    let buffer = Default::default();
    let mut arena = Arena::new();
    let current_buffer = arena.insert(buffer);
    GlobalData {
        buffers: arena,
        current_buffer,
        mode: Mode::Normal,
        cursor: Cursor {
            position: Point { x: 1, y: 1 },
        },
    }
}

fn setup_watcher(msg_sender: Sender<Msg>) -> RecommendedWatcher {
    let (tx, rx) = std::sync::mpsc::channel(); // This is std so that file watcher is happy
    std::thread::spawn(move || {
        for file_event in rx.iter() {
            msg_sender.send(Msg::LibraryEvent(file_event)).unwrap();
        }
    });
    watcher(tx, time::Duration::from_millis(100)).unwrap()
}

fn setup_event_handler(msg_sender: Sender<Msg>) {
    std::thread::spawn(move || {
        use termion::input::TermRead;
        let stdin = std::io::stdin();
        let _stdout = std::io::stdout().into_raw_mode().unwrap(); // Need to turn it into raw mode
        for event in stdin.events() {
            if let Ok(evt) = event {
                msg_sender.send(Msg::StdinEvent(evt)).unwrap();
            }
        }
    });
}

fn setup_external_socket(msg_sender: Sender<Msg>) {
    use std::io::Read;
    use std::os::unix::net::UnixListener;
    std::thread::spawn(move || {
        // Don't care if it did not exist
        let _ = std::fs::remove_file("/tmp/myedit-core");
        let listener = UnixListener::bind("/tmp/myedit-core").unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let cmd: Cmd = rmp_serde::from_read(stream).expect("parsing command");
                    msg_sender
                        .send(Msg::Cmd(cmd))
                        .expect("sending command in message");
                }
                Err(err) => {
                    println!("Error: {}", err);
                    break;
                }
            }
        }
    });
}

fn setup_signals_handler(msg_sender: Sender<Msg>) {
    use signal_hook::iterator::Signals;
    use signal_hook::SIGWINCH;
    let signals = Signals::new(&[SIGWINCH]).unwrap();
    std::thread::spawn(move || {
        for _ in signals.forever() {
            msg_sender.send(Msg::Cmd(Cmd::CleanRender));
        }
    });
}

pub fn start(file: Option<std::path::PathBuf>) {
    let mut global_data = initial_state();
    let utils = utils::build_utils();
    let (msg_sender, msg_receiver) = unbounded::<Msg>();
    let mut watcher = setup_watcher(msg_sender.clone());
    let mut libraries: HashMap<String, DynLib> = load_libs(&mut watcher);

    let mut back_buffer = back_buffer::create_back_buffer();
    if let Some(file) = file {
        msg_sender
            .send(Msg::Cmd(Cmd::LoadFile(file)))
            .expect("loading initial file");
    }
    setup_event_handler(msg_sender.clone());
    setup_external_socket(msg_sender.clone());
    setup_signals_handler(msg_sender.clone());
    println!("{}", termion::clear::All);
    let clone = msg_sender.clone();
    // This is witchcraft to account for channels not liking getting moved across dynamic boundaries :/
    let cmd_handler: Box<Fn(Cmd)> = Box::new(move |msg| clone.send(Msg::Cmd(msg)).unwrap());
    for msg in msg_receiver.iter() {
        match msg {
            Msg::LibraryEvent(ref event) => match event {
                DebouncedEvent::Create(ref path) => {
                    let key = path.file_name().unwrap().to_str().unwrap();
                    libraries.remove(key);
                    let lib = load_lib(path);
                    libraries.insert(key.to_string(), lib);
                }
                _ => {}
            },
            Msg::StdinEvent(ref evt) => {
                use termion::event::{Event, Key};
                match evt {
                    Event::Key(Key::Ctrl('c')) => return,
                    _ => {}
                }
            }
            Msg::Cmd(Cmd::Quit) => {
                return;
            }
            Msg::Cmd(Cmd::CleanRender) => {
                println!("{}", termion::clear::All);
                back_buffer = back_buffer::create_back_buffer();
            }
            _ => {} // handled in libs
        }

        for (_path, lib) in libraries.iter() {
            (*lib.update_fn)(&mut global_data, &msg, &utils, &cmd_handler, lib.data);
        }
        let mut new_back_buffer = back_buffer::create_back_buffer();

        for (_path, lib) in libraries.iter() {
            (*lib.render_fn)(&global_data, &mut new_back_buffer, &utils, lib.data);
        }
        back_buffer::update_stdout(&back_buffer, &new_back_buffer);
        back_buffer = new_back_buffer;
    }
}
