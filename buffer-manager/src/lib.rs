use ropey::Rope;
use types::{Buffer, Cmd, GlobalData, Msg};
#[no_mangle]
pub fn render(global_data: &GlobalData) {}

fn load_buffer(global_data: &mut GlobalData, file_path: std::path::PathBuf) {
    let new_buffer = global_data.buffers.insert(Buffer {
        rope: Rope::from_reader(std::fs::File::open(&file_path).expect("loading file")).expect("building rope"),
        source: file_path,
    });
    global_data.current_buffer = new_buffer;
}

#[no_mangle]
pub fn update(global_data: &mut GlobalData, msg: &Msg) {
    use Cmd::*;
    match msg {
        Msg::Cmd(cmd) => match cmd {
            LoadFile(file_path) => {
                let maybe_index = global_data
                    .buffers
                    .iter()
                    .find(|(_index, buffer)| buffer.source == file_path.as_path())
                    .map(|(index, _buffer)| index);
                if let Some(index) = maybe_index {
                    global_data.current_buffer = index;
                } else {
                    load_buffer(global_data, file_path.clone());
                }
            }
            WriteBuffer(path) => {
                let mut file = std::fs::File::create(path).expect("opening file");
                global_data.buffers[global_data.current_buffer]
                    .rope
                    .write_to(file)
                    .expect("writing to file");
            }
            SearchFiles => {
                use std::process::Command;
                // Command::new("tmux")
                //     .args(&["split-pane", "sk"])
                //     .spawn().unwrap();
                Command::new("tmux")
                    .args(&["split-pane", "sk | xargs -0 -I {} cargo run -- --target test_client --command \"edit {}\""])
                    .spawn().expect("spawning sk in tmux pane");
            }
            _ => {}
        },
        _ => {}
    }
}

#[no_mangle]
pub fn init(global_data: &mut GlobalData) {}
