use ropey::Rope;
use types::{GlobalData, Msg, Buffer};
#[no_mangle]
pub fn render(global_data: &GlobalData) {
}

fn load_buffer (global_data: &mut GlobalData, file_path: String) {
    global_data.buffer = Some(Buffer {
        rope: Rope::from_reader(std::fs::File::open(&file_path).unwrap()).unwrap(),
        source: file_path,
    });
}

#[no_mangle]
pub fn update(global_data: &mut GlobalData, msg: &Msg) {
    use Msg::*;
    match msg {
        LoadFile(file_path) => {
            load_buffer(global_data, file_path.clone());
        },
        _ => {},
    }
}

#[no_mangle]
pub fn init(global_data: &mut GlobalData) {
}
