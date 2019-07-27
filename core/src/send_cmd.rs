use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::os::unix::net::UnixStream;
use types::Cmd;

fn send_over_socket(mut socket: UnixStream, command: Cmd) {
    let mut buf = Vec::new();
    command.serialize(&mut Serializer::new(&mut buf)).unwrap();
    socket.write_all(&buf).unwrap();
}

pub fn send(target: &str, command: &str) {
    let mut chunks = command.split(" ");
    let socket =
        UnixStream::connect(format!("/tmp/myedit-{}", target)).expect("opening socket to write");
    match chunks.next() {
        Some("edit") => {
            if let Some(file) = chunks.next() {
                send_over_socket(socket, Cmd::LoadFile(std::path::PathBuf::from(file.trim())));
            } else {
                panic!("Need to pass file with edit: eg \"edit test.rs\"");
            }
        }
        Some(_) => panic!("Unknown command"),
        None => {}
    }
}