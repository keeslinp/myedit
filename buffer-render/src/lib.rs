use std::ffi::c_void;
use std::sync::mpsc::Sender;

use types::{BackBuffer, ClientIndex, GlobalData, Msg, Point, Rect, Utils};

#[derive(Debug, Default)]
struct Data {
    val: bool,
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
    if let Some(Rect { w: _, h }) = global_data.clients[*client].size {
        //(100, 50);//termion::terminal_size().unwrap();
        let start_line = buffer.start_line;
        let lines_to_render = std::cmp::min(buffer.rope.len_lines() - 1, h as usize - 1);
        for screen_line in 0..lines_to_render {
            let buffer_line = screen_line + start_line;
            let line = buffer.rope.line(buffer_line);
            (utils.write_to_buffer)(
                back_buffer,
                &Point {
                    x: 0,
                    y: screen_line as u16,
                },
                &format!("{}", buffer_line + 1),
                None,
                None,
                None,
            );
            if let Some(line) = line.as_str() {
                (utils.write_to_buffer)(
                    back_buffer,
                    &Point {
                        x: 4,
                        y: screen_line as u16,
                    },
                    line,
                    None,
                    None,
                    None,
                );
            }
        }
    } else {
        (utils.warn)("Missing client size");
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
