use crossbeam_channel::{unbounded, Sender};
use libloading::os::unix::Symbol;
use notify::{watcher, DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::any::Any;
use std::collections::HashMap;
use std::default::Default;
use std::ffi::c_void;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::{fs, path, time};
use termion::raw::IntoRawMode;
use types::{
    BackBuffer, Client, ClientIndex, Cmd, Cursor, GlobalData, Mode, Msg, Point, RemoteCommand,
    Utils,
};

use crate::back_buffer;
use crate::utils;

#[derive(Debug)]
struct DynLib {
    lib: libloading::Library,
    render_fn:
        Symbol<extern "C" fn(&GlobalData, ClientIndex, &mut BackBuffer, &Utils, *mut c_void)>,
    update_fn: Symbol<
        extern "C" fn(&mut GlobalData, &Msg, &Utils, &Box<Fn(ClientIndex, Cmd)>, *mut c_void),
    >,
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
            extern "C" fn(&GlobalData, ClientIndex, &mut BackBuffer, &Utils, *mut c_void),
        > = lib.get(b"render").expect("loading render function");
        let update_fn: libloading::Symbol<
            extern "C" fn(&mut GlobalData, &Msg, &Utils, &Box<Fn(ClientIndex, Cmd)>, *mut c_void),
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
    use slotmap::{SecondaryMap, SlotMap};
    let buffer = Default::default();
    let mut buffer_keys = SlotMap::new();
    let current_buffer = buffer_keys.insert(());
    let mut buffers = SecondaryMap::new();
    buffers.insert(current_buffer, buffer);
    GlobalData {
        buffer_keys,
        buffers,
        cursor: Cursor {
            position: Point { x: 1, y: 1 },
        },
        clients: SecondaryMap::new(),
        client_keys: SlotMap::new(),
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

fn setup_external_socket(msg_sender: Sender<Msg>) {
    std::thread::spawn(move || {
        // Don't care if it did not exist
        let _ = std::fs::remove_file("/tmp/myedit-core");
        let listener = UnixListener::bind("/tmp/myedit-core").unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let RemoteCommand(client, cmd): RemoteCommand =
                        rmp_serde::from_read(stream).expect("parsing command");
                    msg_sender
                        .send(Msg::Cmd(client, cmd))
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

fn handle_client_input(client_index: ClientIndex, stream: UnixStream, msg_sender: Sender<Msg>) {
    std::thread::spawn(move || {
        use termion::event::parse_event;
        let mut bytes = stream.bytes();
        loop {
            let byte = bytes.next();
            if let Some(Ok(byte)) = byte {
                if let Ok(evt) = parse_event(byte, &mut bytes) {
                    msg_sender
                        .send(Msg::StdinEvent(client_index, evt))
                        .expect("sending stdin event from client");
                }
            } else {
                break;
            }
        }
    });
}

fn setup_client_listener(msg_sender: Sender<Msg>) {
    std::thread::spawn(move || {
        let _ = std::fs::remove_file("/tmp/myedit-stdin");
        let listener = UnixListener::bind("/tmp/myedit-stdin").unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    msg_sender.send(Msg::NewClient(stream));
                }
                Err(_) => {
                    break;
                }
            }
        }
    });
}

// TODO: Figure this out
// fn setup_signals_handler(msg_sender: Sender<Msg>) {
//     use signal_hook::iterator::Signals;
//     use signal_hook::SIGWINCH;
//     let signals = Signals::new(&[SIGWINCH]).unwrap();
//     std::thread::spawn(move || {
//         for _ in signals.forever() {
//             msg_sender.send(Msg::Cmd(Cmd::CleanRender));
//         }
//     });
// }

pub fn start(file: Option<std::path::PathBuf>) {
    let mut global_data = initial_state();
    let utils = utils::build_utils();
    let (msg_sender, msg_receiver) = unbounded::<Msg>();
    let mut watcher = setup_watcher(msg_sender.clone());
    let mut libraries: HashMap<String, DynLib> = load_libs(&mut watcher);

    // if let Some(file) = file {
    //     msg_sender
    //         .send(Msg::Cmd(Cmd::LoadFile(file)))
    //         .expect("loading initial file");
    // }
    // setup_event_handler(msg_sender.clone());
    setup_external_socket(msg_sender.clone());
    // setup_signals_handler(msg_sender.clone());
    setup_client_listener(msg_sender.clone());
    println!("{}", termion::clear::All);
    let clone = msg_sender.clone();
    // This is witchcraft to account for channels not liking getting moved across dynamic boundaries :/
    let cmd_handler: Box<Fn(ClientIndex, Cmd)> =
        Box::new(move |client_index, msg| clone.send(Msg::Cmd(client_index, msg)).unwrap());
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
            Msg::StdinEvent(client, ref evt) => {
                use termion::event::{Event, Key};
                match evt {
                    Event::Key(Key::Ctrl('c')) => return,
                    _ => {}
                }
            }
            Msg::Cmd(client_index, Cmd::Quit) => {
                // TODO: don't crash
                global_data.clients[client_index]
                    .stream
                    .shutdown(std::net::Shutdown::Both);
                return;
            }
            Msg::Cmd(client, Cmd::CleanRender) => {
                // TODO: Clean client
                write!(global_data.clients[client].stream, "{}", termion::clear::All);
                global_data.clients[client].back_buffer = back_buffer::create_back_buffer();
            }
            Msg::NewClient(ref stream) => {
                let stream_clone = stream.try_clone().unwrap();
                let client = Client {
                    stream: stream.try_clone().unwrap(),
                    buffer: global_data
                        .buffers
                        .iter()
                        .next()
                        .expect("Getting first buffer")
                        .0,
                    mode: Mode::Normal,
                    back_buffer: back_buffer::create_back_buffer(),
                };
                let index = global_data.client_keys.insert(());
                global_data.clients.insert(index, client);
                handle_client_input(index, stream_clone, msg_sender.clone());
            }
            _ => {} // handled in libs
        }

        for (_path, lib) in libraries.iter() {
            (*lib.update_fn)(&mut global_data, &msg, &utils, &cmd_handler, lib.data);
        }
        if (msg_sender.is_empty()) {
            // Don't bother rendering if there is more in the pipeline
            for client in global_data.client_keys.keys() {
                let mut new_back_buffer = back_buffer::create_back_buffer();
                for (_path, lib) in libraries.iter() {
                    (*lib.render_fn)(&global_data, client, &mut new_back_buffer, &utils, lib.data);
                }
                back_buffer::update_stdout(&global_data.clients[client].back_buffer, &new_back_buffer, global_data.clients[client].stream.try_clone().unwrap());
                global_data.clients[client].back_buffer = new_back_buffer;
            }
        }
    }
}
