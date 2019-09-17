use types::{
    BackBuffer, ClientIndex, Cmd, GlobalData, Msg, Point, Rope, Utils,
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

fn get_char_range_from_points(
    start: &Point,
    end: &Point,
    rope: &Rope,
) -> std::ops::RangeInclusive<usize> {
    let start_index = get_ropey_index_from_point(start, rope);
    let end_index = get_ropey_index_from_point(end, rope);
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
                    rope.remove(get_char_range_from_points(start, end, &rope));
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_first_line_get_ropey_index_from_point() {
        let rope = Rope::from_str(
            "test someting with \
            lots of test data"
        );
        let point = Point {
            x: 5,
            y: 0,
        };
        assert_eq!(get_ropey_index_from_point(&point, &rope), 4);
    }
    #[test]
    fn test_second_line_get_ropey_index_from_point() {
        let rope = Rope::from_str( "test someting with \nlots of test data");
        let point = Point {
            x: 5,
            y: 1,
        };
        assert_eq!(get_ropey_index_from_point(&point, &rope), 24);
    }
    #[test]
    fn test_get_char_range_from_points() {
        let rope = Rope::from_str( "test someting with \nlots of test data");
        let start_point = Point {
            x: 5,
            y: 0,
        };
        let end_point = Point {
            x: 5,
            y: 1,
        };
        assert_eq!(get_char_range_from_points(&start_point, &end_point, &rope), 4..=24);
    }

    #[test]
    fn test_update_delete_char_range() {
        // Definitely abstract a ton of this into another module cause it is a huge PITA to set up
        use types::{Buffer, Client, Mode, Utils};
        use crossbeam_channel::{bounded, Sender};
        let mut global_data = GlobalData::default();
        let rope = Rope::from_str( "test someting with \nlots of test data");
        let new_buffer_index = global_data.buffer_keys.insert(());
        let new_buffer = Buffer {
            rope,
            source: std::path::PathBuf::default(),
            start_line: 0,
        };
        global_data.buffers.insert(new_buffer_index, new_buffer);
        let new_client_index = global_data.client_keys.insert(());
        let new_client = Client {
            stream: std::os::unix::net::UnixStream::pair().unwrap().0,
            back_buffer: std::default::Default::default(),
            buffer: new_buffer_index,
            mode: Mode::Normal,
            size: std::default::Default::default(),
        };
        global_data.clients.insert(new_client_index, new_client);
        let start_point = Point {
            x: 5,
            y: 0,
        };
        let end_point = Point {
            x: 5,
            y: 1,
        };
        let (msg_sender, msg_receiver) = bounded::<Msg>(1);
        let cmd = Cmd::DeleteCharRange(start_point, end_point);
        let msg = Msg::Cmd(new_client_index, cmd);
        let utils = Utils {
            write_to_buffer: |_,_,_,_,_,_| {},
            info: |msg| {},
            warn: |msg|{},
            debug: |msg|{},
            style_range: |_,_,_,_,_,_|{},
            style_rope_slice_range: |_,_,_,_,_,_|{},
        };
        let c_ptr = init();
        let cmd_handler: Box<Fn(ClientIndex, Cmd)> =
            Box::new(move |client_index, msg| msg_sender.send(Msg::Cmd(client_index, msg)).unwrap());
        update(&mut global_data, &msg, &utils, &cmd_handler, c_ptr);
        match msg_receiver.try_recv() {
            Ok(val) => {
                match val {
                    Msg::Cmd(client_index, cmd) => {
                        assert_eq!(client_index, new_client_index);
                        assert_eq!(cmd, Cmd::BufferModified);
                    },
                    other => {
                        panic!("Wrong message type sent: {:?}", other);
                    }
                }
            },
            Err(_) => panic!("There was no value waiting in the reciever")
        }
        assert_eq!(String::from(global_data.buffers[new_buffer_index].rope.clone()).as_str(), "testof test data");
    }
}
