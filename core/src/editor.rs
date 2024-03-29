use crossbeam_channel::{unbounded, Sender};
use libloading::os::unix::Symbol;
use notify::{watcher, DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};

use log::{info, LevelFilter};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::collections::HashMap;
use std::default::Default;
use std::ffi::c_void;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::{fs, path, time};

use types::{
    BackBuffer, Client, ClientIndex, Cmd, GlobalData, InitializeClient, Mode, Msg, Rect,
    RemoteCommand, Utils,
};

use crate::back_buffer;
use crate::utils;

#[derive(Debug)]
struct DynLib {
    lib: libloading::Library,
    render_fn:
        Symbol<extern "C" fn(&GlobalData, &ClientIndex, &mut BackBuffer, &Utils, *mut c_void)>,
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

fn load_lib(path: &path::PathBuf, global_data: &GlobalData) -> DynLib {
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
            extern "C" fn(&GlobalData, &ClientIndex, &mut BackBuffer, &Utils, *mut c_void),
        > = lib.get(b"render").expect("loading render function");
        let update_fn: libloading::Symbol<
            extern "C" fn(&mut GlobalData, &Msg, &Utils, &Box<Fn(ClientIndex, Cmd)>, *mut c_void),
        > = lib.get(b"update").expect("loading update function");
        let init_fn: libloading::Symbol<extern "C" fn(&GlobalData) -> *mut c_void> =
            lib.get(b"init").expect("loading init function");
        let cleanup_fn: libloading::Symbol<extern "C" fn(*mut c_void)> =
            lib.get(b"cleanup").expect("loading cleanup function");
        let data = init_fn(global_data);
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
// const LIB_LOC: &'static str = "./target/debug";
const LIB_LOC: &'static str = "./target/release";
// #[cfg(not(debug_assertion))]

fn load_libs(
    watcher: &mut RecommendedWatcher,
    global_data: &GlobalData,
) -> HashMap<String, DynLib> {
    use std::fs::read_dir;
    read_dir(LIB_LOC)
        .expect("reading lib folder")
        .map(|dir_buff| dir_buff.unwrap().path())
        .filter(|path| path.extension().map(|ext| ext == "dylib").unwrap_or(false))
        .inspect(|path| watcher.watch(path, RecursiveMode::NonRecursive).unwrap())
        .map(|path| {
            (
                path.file_name().unwrap().to_str().unwrap().to_owned(),
                load_lib(&path, global_data),
            )
        })
        .collect()
}

fn initial_state() -> GlobalData {
    use types::{SecondaryMap, SlotMap};
    let buffer = Default::default();
    let mut buffer_keys = SlotMap::new();
    let current_buffer = buffer_keys.insert(());
    let mut buffers = SecondaryMap::new();
    buffers.insert(current_buffer, buffer);
    GlobalData {
        buffer_keys,
        buffers,
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
                Ok(stream) => {
                    let RemoteCommand(client, cmd): RemoteCommand =
                        rmp_serde::from_read(stream).expect("parsing command");
                    msg_sender
                        .send(Msg::Cmd(client, cmd))
                        .expect("sending command in message");
                }
                Err(_err) => {
                    // println!("Error: {}", err);
                    break;
                }
            }
        }
    });
}

fn handle_client_input(client_index: ClientIndex, stream: UnixStream, msg_sender: Sender<Msg>) {
    std::thread::spawn(move || {
        use termion::input::TermRead;
        for event in stream.events() {
            msg_sender
                .send(Msg::StdinEvent(client_index, event.unwrap()))
                .expect("sending stdin event from client");
        }
    });
}

fn setup_client_listener(msg_sender: Sender<Msg>) {
    std::thread::spawn(move || {
        let _ = std::fs::remove_file("/tmp/myedit-stdin");
        let listener = UnixListener::bind("/tmp/myedit-stdin").unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    msg_sender.send(Msg::NewClient(stream));
                }
                Err(_) => {
                    break;
                }
            }
        }
    });
}

fn setup_logging() {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .append(false)
        .build("output.log")
        .expect("logging setup");
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();
    log4rs::init_config(config).unwrap();
    info!("Hello World");
}

pub fn start(file: Option<std::path::PathBuf>) {
    setup_logging();
    let mut global_data = initial_state();
    let utils = utils::build_utils();
    let (msg_sender, msg_receiver) = unbounded::<Msg>();
    let mut watcher = setup_watcher(msg_sender.clone());
    let mut libraries: HashMap<String, DynLib> = load_libs(&mut watcher, &global_data);

    setup_external_socket(msg_sender.clone());
    setup_client_listener(msg_sender.clone());
    let clone = msg_sender.clone();
    // This is witchcraft to account for channels not liking getting moved across dynamic boundaries :/
    let cmd_handler: Box<Fn(ClientIndex, Cmd)> =
        Box::new(move |client_index, msg| clone.send(Msg::Cmd(client_index, msg)).unwrap());
    for msg in msg_receiver.iter() {
        info!("message ->{:?}", msg);
        if let Msg::Cmd(ref client, _) = msg {
            // If the client quit don't do anything
            if !global_data.client_keys.contains_key(*client) {
                info!("ignoring message becasue client is gone");
                continue;
            }
        }
        match msg {
            Msg::LibraryEvent(ref event) => match event {
                DebouncedEvent::Create(ref path) => {
                    let key = path.file_name().unwrap().to_str().unwrap();
                    libraries.remove(key);
                    let lib = load_lib(path, &global_data);
                    libraries.insert(key.to_string(), lib);
                    info!("Reloaded lib: {}", &key);
                }
                _ => {}
            },
            Msg::StdinEvent(_client, ref evt) => {
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
                global_data.clients.remove(client_index);
                global_data.client_keys.remove(client_index);
                // Don't want to have other libs try to run this event
                continue;
            }
            Msg::Cmd(_client, Cmd::Kill) => {
                std::fs::remove_file("/tmp/myedit-core");
                std::fs::remove_file("/tmp/myedit-stdin");
                return;
            }
            Msg::Cmd(client, Cmd::CleanRender) => {
                write!(
                    global_data.clients[client].stream,
                    "{}",
                    termion::clear::All
                );
                global_data.clients[client].back_buffer = back_buffer::create_back_buffer(
                    global_data.clients[client]
                        .size
                        .clone()
                        .unwrap_or(Rect::default()),
                );
            }
            Msg::NewClient(ref stream) => {
                let stream_clone = stream.try_clone().unwrap();
                let mut client = Client {
                    stream: stream.try_clone().unwrap(),
                    buffer: global_data
                        .buffers
                        .iter()
                        .next()
                        .expect("Getting first buffer")
                        .0,
                    mode: Mode::Normal,
                    back_buffer: back_buffer::create_back_buffer(Rect::default()),
                    size: None,
                };
                let index = global_data.client_keys.insert(());
                // Tell the client who they are
                info!("Information client {:?}", index);
                let mut buf = Vec::new();
                use serde::ser::Serialize;
                InitializeClient(index)
                    .serialize(&mut rmp_serde::Serializer::new(&mut buf))
                    .unwrap();
                client.stream.write_all(&buf).expect("sending client index");
                client.stream.flush().expect("flushing stream");
                // Store the client
                global_data.clients.insert(index, client);
                handle_client_input(index, stream_clone, msg_sender.clone());
                if let Some(ref file) = file {
                    msg_sender
                        .send(Msg::Cmd(index, Cmd::LoadFile(file.to_path_buf())))
                        .expect("loading initial file");
                }
                info!("Client {:?} initialized", index);
            }
            Msg::Cmd(client, Cmd::ResizeClient(ref new_dim)) => {
                global_data.clients[client].size = Some(new_dim.clone());
                msg_sender.send(Msg::Cmd(client, Cmd::CleanRender));
            }
            _ => {} // handled in libs
        }

        for (path, lib) in libraries.iter() {
            info!("updating: {}", path);
            (*lib.update_fn)(&mut global_data, &msg, &utils, &cmd_handler, lib.data);
        }
        if msg_sender.is_empty() {
            // Don't bother rendering if there is more in the pipeline
            for client in global_data.client_keys.keys() {
                if let Some(size) = global_data.clients[client].size.clone() {
                    let mut new_back_buffer = back_buffer::create_back_buffer(size);
                    for (path, lib) in libraries.iter() {
                        info!("rendering: {}", path);
                        (*lib.render_fn)(
                            &global_data,
                            &client,
                            &mut new_back_buffer,
                            &utils,
                            lib.data,
                        );
                        info!("rendered");
                    }
                    back_buffer::update_stdout(
                        &global_data.clients[client].back_buffer,
                        &new_back_buffer,
                        global_data.clients[client].stream.try_clone().unwrap(),
                    );
                    global_data.clients[client].back_buffer = new_back_buffer;
                }
            }
        }
    }
}
