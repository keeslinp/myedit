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
use types::{BackBuffer, Cmd, Cursor, GlobalData, Mode, Msg, Point, Utils};

fn setup_stdin(mut stream: UnixStream) {
    std::thread::spawn(move || {
        use termion::input::TermRead;
        let stdin = std::io::stdin();
        let _stdout = std::io::stdout().into_raw_mode().unwrap(); // Need to turn it into raw mode
        let lock = stdin.lock();
        for byte in lock.bytes() {
            stream.write(&[byte.unwrap()]);
        }
    });
}

fn setup_stdout(mut stream: UnixStream) {
    std::thread::spawn(move || {
        let stdout = std::io::stdout();
        let mut lock = stdout.lock();
        for byte in stream.bytes() {
            write!(lock, "test");
            lock.write_all(&[byte.unwrap()]);
        }
    });
}

fn setup_external_socket() -> UnixStream {
    UnixStream::connect("/tmp/myedit-stdin").expect("setting up socket")
}

fn setup_signals_handler(quit: Sender<()>) {
    use signal_hook::iterator::Signals;
    use signal_hook::SIGWINCH;
    let signals = Signals::new(&[SIGWINCH]).unwrap();
    std::thread::spawn(move || {
        for _ in signals.forever() {
            // quit.send(());
        }
    });
}

fn launch_core(file: Option<std::path::PathBuf>) {
    use std::process::Command;
    Command::new("cargo")
        .args(&["run", "--", "test_file.rs", "--core"])
        .spawn();
}

pub fn start(file: Option<std::path::PathBuf>) {
    launch_core(file);
    let stream = setup_external_socket();
    let (tx, rx) = unbounded();
    setup_signals_handler(tx);
    setup_stdin(stream.try_clone().unwrap());
    setup_stdout(stream.try_clone().unwrap());
    let _ = rx.recv();
    println!("all done");
}
