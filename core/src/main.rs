use crossbeam_channel::{unbounded, Receiver, Sender};
use libloading::os::unix::Symbol;
use notify::{watcher, DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::default::Default;
use std::{fs, path, time};
use termion::raw::IntoRawMode;
use types::{BackBuffer, Cursor, GlobalData, Mode, Msg, Point, Utils};
mod back_buffer;
mod utils;

#[derive(Debug)]
struct DynLib {
    lib: libloading::Library,
    render_fn: Symbol<extern "C" fn(&GlobalData, &mut BackBuffer, &Utils)>,
    update_fn: Symbol<extern "C" fn(&mut GlobalData, &Msg, &Utils, &Box<Fn(Msg)>)>,
    init_fn: Symbol<extern "C" fn(&mut GlobalData, &Utils)>,
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
        let render_fn: libloading::Symbol<extern "C" fn(&GlobalData, &mut BackBuffer, &Utils)> =
            lib.get(b"render").expect("loading render function");
        let update_fn: libloading::Symbol<
            extern "C" fn(&mut GlobalData, &Msg, &Utils, &Box<Fn(Msg)>),
        > = lib.get(b"update").expect("loading update function");
        let init_fn: libloading::Symbol<extern "C" fn(&mut GlobalData, &Utils)> =
            lib.get(b"init").expect("loading init function");
        DynLib {
            render_fn: render_fn.into_raw(),
            update_fn: update_fn.into_raw(),
            init_fn: init_fn.into_raw(),
            lib,
        }
    }
}

fn load_libs(watcher: &mut RecommendedWatcher) -> HashMap<String, DynLib> {
    use std::fs::read_dir;
    read_dir("./target/debug")
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
    GlobalData {
        buffer: Default::default(),
        command_buffer: Default::default(),
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
        let stdout = std::io::stdout().into_raw_mode().unwrap();
        for event in stdin.events() {
            if let Ok(evt) = event {
                msg_sender.send(Msg::StdinEvent(evt)).unwrap();
            }
        }
    });
}

use structopt::StructOpt;
use std::path::PathBuf;
#[derive(Debug, StructOpt)]
#[structopt(name = "myedit", about = "Personalized text editor and dev environment")]
struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: std::path::PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    let mut global_data = initial_state();
    let utils = utils::build_utils();
    let (msg_sender, msg_receiver) = unbounded::<Msg>();
    let mut watcher = setup_watcher(msg_sender.clone());
    let mut libraries: HashMap<String, DynLib> = load_libs(&mut watcher);

    let mut back_buffer = back_buffer::create_back_buffer();
    msg_sender.send(Msg::LoadFile(opt.input));
    setup_event_handler(msg_sender.clone());
    println!("{}", termion::clear::All);
    let clone = msg_sender.clone();
    // This is witchcraft to account for channels not liking getting moved across dynamic boundaries :/
    let msg_handler: Box<Fn(Msg)> = Box::new(move |msg| clone.send(msg).unwrap());
    for msg in msg_receiver.iter() {
        use Msg::*;
        match msg {
            LibraryEvent(ref event) => match event {
                DebouncedEvent::Create(ref path) => {
                    let key = path.file_name().unwrap().to_str().unwrap();
                    libraries.remove(key);
                    let lib = load_lib(path);
                    (*lib.init_fn)(&mut global_data, &utils);
                    libraries.insert(key.to_string(), lib);
                }
                _ => {}
            },
            StdinEvent(ref evt) => {
                use termion::event::{Event, Key};
                match evt {
                    Event::Key(Key::Ctrl('c')) => return,
                    _ => {}
                }
            }
            Quit => {
                return;
            }
            _ => {} // handled in libs
        }

        for (path, lib) in libraries.iter() {
            (*lib.update_fn)(&mut global_data, &msg, &utils, &msg_handler);
        }
        // for (path, lib) in libraries.iter() {
        //     (*lib.update_fn)(&mut global_data, &msg, &utils, other_send.clone());
        // }
        let mut new_back_buffer = back_buffer::create_back_buffer();

        for (_path, lib) in libraries.iter() {
            (*lib.render_fn)(&global_data, &mut new_back_buffer, &utils);
        }
        back_buffer::update_stdout(&back_buffer, &new_back_buffer);
        back_buffer = new_back_buffer;
    }
}
