use types::{
    BackBuffer, Buffer, BufferIndex, Client, ClientIndex, Cmd, Color, DeleteDirection, Direction,
    GlobalData, JumpType, Mode, Msg, Point, Rect, Rope, SecondaryMap, Utils,
};

#[derive(Debug, Default)]
struct State {
    value: bool, // This just makes it possible to build
}

#[no_mangle]
pub fn render(
    _global_data: &GlobalData,
    _client: &ClientIndex,
    _back_buffer: &mut BackBuffer,
    _utils: &Utils,
    _data_ptr: *mut c_void,
) {
}

fn get_ropey_index_from_point(position: &Point, rope: &Rope) -> usize {
    rope.line_to_char(position.y as usize) + position.x as usize - 1
}

fn get_char_range(
    position: &Point,
    selection_anchor: &Point,
    rope: &Rope,
) -> std::ops::RangeInclusive<usize> {
    let start_index = get_ropey_index_from_point(position, rope);
    let end_index = get_ropey_index_from_point(selection_anchor, rope);
    if start_index < end_index {
        start_index..=end_index
    } else {
        end_index..=start_index
    }
}

#[no_mangle]
pub fn update(
    global_data: &mut GlobalData,
    cmd: &Msg,
    _utils: &Utils,
    send_cmd: &Box<Fn(ClientIndex, Cmd)>,
    _data_ptr: *mut c_void,
) {
    // let _data: Box<State> = unsafe { Box::from_raw(data_ptr as *mut State) };
    use Cmd::*;
    match cmd {
        Msg::Cmd(client_index, cmd) => {
            let client = &global_data.clients[*client_index];
            let current_buffer = &mut global_data.buffers[client.buffer];
            let rope = &mut current_buffer.rope;
            match cmd {
                InsertCharAtPoint(c, point) => {
                    let index = get_ropey_index_from_point(point, &rope);
                    rope.insert_char(index, *c);
                    send_cmd(*client_index, BufferModified);
                }
                InsertStringAtPoint(string, point) => {
                    let index = get_ropey_index_from_point(point, &rope);
                    rope.insert(index, &string);
                    send_cmd(*client_index, BufferModified);
                }
                DeleteCharRange(start, end) => {
                    rope.remove(get_char_range(start, end, &rope));
                    send_cmd(*client_index, BufferModified);
                }
                _ => {}
            }
        }
        _ => {}
    };
    // std::mem::forget(data);
}

use std::ffi::c_void;

#[no_mangle]
pub fn init() -> *mut c_void {
    Box::into_raw(Box::new(State::default())) as *mut c_void
}

#[no_mangle]
pub fn cleanup(data: *mut c_void) {
    unsafe {
        let ptr = Box::from_raw(data);
        drop(ptr);
    }
}
