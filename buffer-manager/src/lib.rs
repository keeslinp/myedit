use ropey::Rope;
use types::{Buffer, Cmd, GlobalData, Msg, ClientIndex, KeyData, Utils};
#[no_mangle]
pub fn render(global_data: &GlobalData) {}

fn load_buffer(global_data: &mut GlobalData, client: ClientIndex, file_path: std::path::PathBuf) {
    let buffer_key = global_data.buffer_keys.insert(());
    global_data.buffers.insert(buffer_key, Buffer {
        rope: Rope::from_reader(std::fs::File::open(&file_path).expect("loading file"))
            .expect("building rope"),
        source: file_path,
        start_line: 0,
    });
    global_data.clients[client].buffer = buffer_key;
}

#[no_mangle]
pub fn update(global_data: &mut GlobalData, msg: &Msg, utils: &Utils) {
    use Cmd::*;
    match msg {
        Msg::Cmd(ref client, cmd) => match cmd {
            LoadFile(file_path) => {
                (utils.info)(format!("Loading buffer: {}", file_path.to_str().unwrap_or("invalid file")));
                let maybe_index = global_data
                    .buffers
                    .iter()
                    .find(|(_index, buffer)| buffer.source == file_path.as_path())
                    .map(|(index, _buffer)| index);
                if let Some(index) = maybe_index {
                    global_data.clients[*client].buffer = index;
                } else {
                    load_buffer(global_data, *client, file_path.clone());
                }
            }
            WriteBuffer(path) => {
                let mut file = std::fs::File::create(path).expect("opening file");
                global_data.buffers[global_data.clients[*client].buffer]
                    .rope
                    .write_to(file)
                    .expect("writing to file");
            }
            SearchFiles => {
                use std::process::Command;
                let key_data: KeyData = KeyData::from(client.clone());
                Command::new("tmux")
                    .args(&["split-pane", &format!("sk | xargs -0 -I {{}} cargo run --release -- --target {} --command \"edit {{}}\"", key_data.as_ffi())])
                    .spawn().expect("spawning sk in tmux pane");
            }
            _ => {}
        },
        _ => {}
    }
}

use std::ffi::c_void;
#[no_mangle]
pub fn init() -> *mut c_void {
    0 as *mut c_void
}

#[no_mangle]
pub fn cleanup(data: *mut c_void) {
    unsafe {
        let ptr = Box::from_raw(data);
        drop(ptr);
    }
}
