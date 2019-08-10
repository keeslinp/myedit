use rmp_serde::{Serializer};
use serde::{Serialize};
use std::io::Write;
use std::os::unix::net::UnixStream;
use types::{Cmd, RemoteCommand, ClientIndex, KeyData};

fn send_over_socket(mut socket: UnixStream, client: ClientIndex, command: Cmd) {
    let mut buf = Vec::new();
    RemoteCommand(client, command).serialize(&mut Serializer::new(&mut buf)).unwrap();
    socket.write_all(&buf).unwrap();
    socket.flush().unwrap();
}

pub fn send(target: u64, command: &str) {
    let mut chunks = command.split(" ");
    let socket = UnixStream::connect(format!("/tmp/myedit-core")).expect("opening socket to write");
    let client = ClientIndex::from(KeyData::from_ffi(target));
    match chunks.next() {
        Some("edit") => {
            if let Some(file) = chunks.next() {
                send_over_socket(socket, client, Cmd::LoadFile(std::path::PathBuf::from(file.trim())));
            } else {
                panic!("Need to pass file with edit: eg \"edit test.rs\"");
            }
        }
        Some(_) => panic!("Unknown command"),
        None => {}
    }
}
