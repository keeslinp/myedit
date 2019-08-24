use types::{
    BackBuffer, Buffer, BufferIndex, Client, ClientIndex, Cmd, Color, DeleteDirection, Direction,
    GlobalData, JumpType, Mode, Msg, Point, Rect, Rope, SecondaryMap, Utils,
};

#[derive(Debug, Default)]
struct State {
    register: String,
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

#[no_mangle]
pub fn update(
    global_data: &mut GlobalData,
    cmd: &Msg,
    _utils: &Utils,
    send_cmd: &Box<Fn(ClientIndex, Cmd)>,
    data_ptr: *mut c_void,
) {
    let mut data: Box<State> = unsafe { Box::from_raw(data_ptr as *mut State) };
    use Cmd::*;
    match cmd {
        Msg::Cmd(client_index, cmd) => {
            match cmd {
                YankValue(val) => {
                    data.register = val.clone();
                }
                PasteAtPoint(position) => {
                    send_cmd(*client_index, InsertStringAtPoint(data.register.clone(), position.clone()));
                }
                _ => {}
            }
        }
        _ => {}
    };
    std::mem::forget(data);
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
