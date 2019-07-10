use libloading::os::unix::Symbol;
use notify::{watcher, DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::{
    fs, path,
    sync::mpsc::{channel, Sender},
    thread, time,
};
use types::{GlobalData, Msg, Mode};
use ropey::Rope;
use termion::raw::IntoRawMode;

#[derive(Debug)]
struct DynLib {
    lib: libloading::Library,
    render_fn: Symbol<extern "C" fn(&GlobalData)>,
    update_fn: Symbol<extern "C" fn(&mut GlobalData, &Msg)>,
    init_fn: Symbol<extern "C" fn(&mut GlobalData)>,
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
        let render_fn: libloading::Symbol<extern "C" fn(&GlobalData)> =
            lib.get(b"render").expect("loading render function");
        let update_fn: libloading::Symbol<extern "C" fn(&mut GlobalData, &Msg)> =
            lib.get(b"update").expect("loading update function");
        let init_fn: libloading::Symbol<extern "C" fn(&mut GlobalData)> =
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
        buffer: None,
        mode: Mode::Normal,
    }
}

fn setup_watcher(msg_sender: Sender<Msg>) -> RecommendedWatcher {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        for file_event in rx.iter() {
            msg_sender.send(Msg::LibraryEvent(file_event)).unwrap();
        }
    });
    watcher(tx, time::Duration::from_secs(1)).unwrap()
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

fn main() {
    let mut global_data = initial_state();
    let (msg_sender, msg_receiver) = channel::<Msg>();
    let mut watcher = setup_watcher(msg_sender.clone());
    let mut libraries: HashMap<String, DynLib> = load_libs(&mut watcher);
    msg_sender.send(Msg::LoadFile("test_file.rs".into()));
    setup_event_handler(msg_sender.clone());
    for msg in msg_receiver.iter() {
        use Msg::*;
        match msg {
            LibraryEvent(ref event) => match event {
                DebouncedEvent::Create(ref path) => {
                    let key = path.file_name().unwrap().to_str().unwrap();
                    libraries.remove(key);
                    let lib = load_lib(path);
                    (*lib.init_fn)(&mut global_data);
                    libraries.insert(key.to_string(), lib);
                }
                _ => {}
            },
            StdinEvent(ref evt) => {
                use termion::event::{Event, Key};
                match evt {
                    Event::Key(Key::Ctrl('c')) => return,
                    _ => {},
                }

            }
            _ => {} // handled in libs
        }

        for (_path, lib) in libraries.iter() {
            (*lib.update_fn)(&mut global_data, &msg);
        }
        for (_path, lib) in libraries.iter() {
            (*lib.render_fn)(&global_data);
        }
    }
}
