use types::{GlobalData, Msg, Utils, Point, BackBuffer};


#[no_mangle]
pub fn render(global_data: &GlobalData, back_buffer: &mut BackBuffer, utils: &Utils) {
    if let Some(ref buffer) = global_data.buffer {
        for (index, line) in buffer.rope.lines().enumerate() {
            (utils.write_to_buffer)(back_buffer, &Point { x: 0, y: index as u16 }, line.as_str().unwrap_or(""), None, None, None);
            // write!(lock, "{}{}{}", termion::cursor::Goto(1, index as u16), termion::clear::CurrentLine, line.as_str().unwrap_or(""));
        }
    }
}

#[no_mangle]
pub fn update(_global_data: &GlobalData, _msg: &Msg, _utils: &Utils) {}

#[no_mangle]
pub fn init(_global_data: &mut GlobalData) {
}
