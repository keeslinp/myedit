use crossbeam_channel::{unbounded, bounded, Sender};
use notify::{Watcher};
use std::io::{Read, Write};
use std::os::unix::net::{UnixStream};
use termion::raw::IntoRawMode;

fn setup_stdin(mut stream: UnixStream) {
    std::thread::spawn(move || {
        
        let stdin = std::io::stdin();
        let _stdout = std::io::stdout().into_raw_mode().unwrap(); // Need to turn it into raw mode
        let lock = stdin.lock();
        for byte in lock.bytes() {
            stream.write(&[byte.unwrap()]);
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

fn setup_signals_handler(_quit: Sender<()>) {
    use signal_hook::iterator::Signals;
    use signal_hook::SIGWINCH;
    let signals = Signals::new(&[SIGWINCH]).unwrap();
    std::thread::spawn(move || {
        for _ in signals.forever() {
        }
    });
}

fn launch_core(file: Option<std::path::PathBuf>) {
    use std::process::{Command, Stdio };
    Command::new("cargo")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .args(&["run", &file.and_then(|val| val.to_str().map(|s| s.to_owned())).unwrap_or(String::new()), "--core"])
        .spawn();
}

pub fn start(file: Option<std::path::PathBuf>) {
    launch_core(file);
    std::thread::sleep(std::time::Duration::from_secs(1));
    let stream = setup_external_socket();
    let (tx, rx) = bounded(1);
    setup_signals_handler(tx.clone());
    setup_stdin(stream.try_clone().unwrap());
    setup_stdout(stream.try_clone().unwrap(), tx);
    let _ = rx.recv();
    println!("all done");
}
