use ropey::Rope;
use types::{Buffer, GlobalData, Msg};
#[no_mangle]
pub fn render(global_data: &GlobalData) {}

fn load_buffer(global_data: &mut GlobalData, file_path: std::path::PathBuf) {
    let new_buffer = global_data.buffers.insert(Buffer {
        rope: Rope::from_reader(std::fs::File::open(&file_path).unwrap()).unwrap(),
        source: file_path,
    });
    global_data.current_buffer = new_buffer;
}

#[no_mangle]
pub fn update(global_data: &mut GlobalData, msg: &Msg) {
    use Msg::*;
    match msg {
        LoadFile(file_path) => {
            let maybe_index = global_data.buffers.iter().find(|(_index, buffer)| buffer.source == file_path.as_path()).map(|(index, _buffer)| index);
            if let Some(index) = maybe_index {
                global_data.current_buffer = index;
            } else {
                load_buffer(global_data, file_path.clone());
            }
        },
        WriteBuffer(path) => {
            let mut file = std::fs::File::create(path).expect("opening file");
            global_data.buffers[global_data.current_buffer].rope.write_to(file).expect("writing to file");
        },
        _ => {}
    }
}

#[no_mangle]
pub fn init(global_data: &mut GlobalData) {}
