use std::sync::mpsc::Sender;
use types::{BackBuffer, GlobalData, Msg, Point, Utils};

#[no_mangle]
pub fn render(global_data: &GlobalData, back_buffer: &mut BackBuffer, utils: &Utils) {
    for (index, line) in global_data.buffer.rope.lines().enumerate() {
        (utils.write_to_buffer)(
            back_buffer,
            &Point {
                x: 0,
                y: index as u16,
            },
            line.as_str().unwrap_or(""),
            None,
            None,
            None,
        );
    }
}

#[no_mangle]
pub fn update(_global_data: &GlobalData, _msg: &Msg, _utils: &Utils, _msg_sender: &Sender<Msg>) {}

#[no_mangle]
pub fn init(_global_data: &mut GlobalData) {}
