use crossbeam_channel::{bounded, Sender};
use notify::Watcher;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use termion::raw::IntoRawMode;
use types::{ClientIndex, Cmd, InitializeClient, Rect};

fn setup_stdin(mut stream: UnixStream) {
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let _stdout = std::io::stdout().into_raw_mode().unwrap(); // Need to turn it into raw mode
        let lock = stdin.lock();
        for byte in lock.bytes() {
            stream.write(&[byte.expect("reading bytes")]);
        }
    });
}

fn setup_stdout(stream: UnixStream, quit: Sender<()>) {
    std::thread::spawn(move || {
        let stdout = std::io::stdout();
        let mut lock = stdout.lock();
        write!(lock, "{}", termion::clear::All);
        lock.flush().unwrap();
        for byte in stream.bytes() {
            // write!(lock, "read")
            lock.write_all(&[byte.unwrap()]);
            lock.flush().unwrap();
        }
        quit.send(()).unwrap();
    });
}

fn setup_external_socket() -> UnixStream {
    UnixStream::connect("/tmp/myedit-stdin").expect("setting up socket")
}

fn send_size_to_editor(client: ClientIndex) {
    use crate::send_cmd::send_over_socket;
    let (w, h) = termion::terminal_size().expect("getting terminal size");
    let command_stream = setup_command_socket();
    send_over_socket(&command_stream, client, Cmd::ResizeClient(Rect { w, h }));
}

fn setup_signals_handler(client: ClientIndex) {
    use signal_hook::iterator::Signals;
    use signal_hook::SIGWINCH;
    let signals = Signals::new(&[SIGWINCH]).unwrap();
    std::thread::spawn(move || {
        for _ in signals.forever() {
            send_size_to_editor(client);
        }
    });
}

fn launch_core(file: Option<std::path::PathBuf>) -> bool {
    use std::process::{Command, Stdio};
    if !std::path::Path::new("/tmp/myedit-stdin").exists() {
        Command::new("cargo")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .args(&[
                "run",
                "--release",
                &file
                    .and_then(|val| val.to_str().map(|s| s.to_owned()))
                    .unwrap_or(String::new()),
                "--core",
            ])
            .spawn();
        true
    } else {
        false
    }
}

fn setup_command_socket() -> UnixStream {
    UnixStream::connect("/tmp/myedit-core").expect("opening socket to write commands")
}

pub fn get_client_index(stream: &UnixStream) -> ClientIndex {
    println!("About to read");
    let InitializeClient(client_index) =
        rmp_serde::from_read(stream).expect("parsing client initialize");
    println!("got it: {:?}", client_index);
    client_index
}

pub fn start(file: Option<std::path::PathBuf>) {
    if launch_core(file) {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    let stream = setup_external_socket();
    // let command_stream = setup_command_socket();
    let (tx, rx) = bounded(1);
    let client_index = get_client_index(&stream);
    send_size_to_editor(client_index);
    setup_stdin(stream.try_clone().unwrap());
    setup_stdout(stream.try_clone().unwrap(), tx);
    setup_signals_handler(client_index);
    let _ = rx.recv();
    println!("all done");
}
