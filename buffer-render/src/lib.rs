use std::ffi::c_void;
use std::sync::mpsc::Sender;

use types::{BackBuffer, ClientIndex, Color, GlobalData, Msg, Point, Utils};

#[derive(Debug)]
struct Data {
    val: bool,
}

impl Default for Data {
    fn default() -> Data {
        Data { val: false }
    }
}

fn color_from_syntect_color(color: &syntect::highlighting::Color) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
    }
}

#[no_mangle]
pub fn render(
    global_data: &GlobalData,
    client: &ClientIndex,
    back_buffer: &mut BackBuffer,
    utils: &Utils,
    data_ptr: *mut c_void,
) {
    let data: Box<Data> = unsafe { Box::from_raw(data_ptr as *mut Data) };
    let buffer = &global_data.buffers[global_data.clients[*client].buffer];
    let (_cols, rows) = (100, 50);//termion::terminal_size().unwrap();
    for (index, line) in buffer
        .rope
        .lines()
        .skip(buffer.start_line)
        .take(rows as usize - 1)
        .enumerate()
    {
        // eprintln!("{}", line.len_chars());
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
    std::mem::forget(data);
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
