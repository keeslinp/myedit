use std::sync::mpsc::Sender;
use types::{BackBuffer, GlobalData, Msg, Point, Utils};
use std::ffi::c_void;

#[derive(Debug, Default)]
struct Data {
    value: u8,
}

#[no_mangle]
pub fn render(global_data: &GlobalData, back_buffer: &mut BackBuffer, utils: &Utils) {
    let buffer = &global_data.buffers[global_data.current_buffer];
    let (cols, rows) = termion::terminal_size().unwrap();
    for (index, line) in buffer
        .rope
        .lines()
        .skip(buffer.start_line)
        .take(rows as usize - 1)
        .enumerate()
    {
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
pub fn init() -> *mut c_void {
    unsafe { Box::into_raw(Box::new(Data::default())) as *mut c_void }
}

#[no_mangle]
pub fn cleanup(data: *mut c_void) {
    unsafe {
        let ptr = Box::from_raw(data);
        drop(ptr);
    }
}
