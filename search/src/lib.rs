use std::ffi::c_void;

use termion::cursor::{Goto, Show};
use types::{
    BackBuffer, ClientIndex, Cmd, DeleteDirection, Direction, GlobalData, Mode, Msg, Point, Rect,
    Rope, SecondaryMap, Utils,
};

#[derive(Debug, Default)]
struct SearchBuffer {
    text: String,
    index: usize,
}

#[derive(Debug, Default)]
struct SearchResult {
    current_index: usize,
}

#[derive(Debug, Default)]
struct Data {
    search_buffer: Option<SearchBuffer>,
    search_results: SecondaryMap<ClientIndex, SearchResult>,
}

#[no_mangle]
pub fn render(
    global_data: &GlobalData,
    client: &ClientIndex,
    back_buffer: &mut BackBuffer,
    utils: &Utils,
    data_ptr: *mut c_void,
) {
    let mode = &global_data.clients[*client].mode;
    let data = unsafe { Box::from_raw(data_ptr as *mut Data) };
    if let Some(Rect { w: _, h }) = global_data.clients[*client].size {
        let status_row_y = h - 1;
        if let Some(ref search_buffer) = data.search_buffer {
            if *mode == Mode::Search {
                (utils.write_to_buffer)(
                    back_buffer,
                    &Point {
                        x: 0,
                        y: status_row_y,
                    },
                    &format!("/{}", search_buffer.text),
                    None,
                    None,
                    None,
                );
                use std::io::Write;
                let mut stream = global_data.clients[*client].stream.try_clone().unwrap();
                write!(
                    stream,
                    "{}{}",
                    Show,
                    Goto(search_buffer.index as u16 + 2, status_row_y + 1)
                );
            }
        }
    }
    // print!("{}{} {:?} {}", style::Invert, Goto(cols - 10, rows), global_data.mode, style::NoInvert);
    std::mem::forget(data);
}

fn new_search_result(query: &str, rope: &Rope) -> Option<SearchResult> {
    None
}

#[no_mangle]
pub fn update(
    global_data: &mut GlobalData,
    msg: &Msg,
    _utils: &Utils,
    send_cmd: &Box<Fn(ClientIndex, Cmd)>,
    data_ptr: *mut c_void,
) {
    let mut data = unsafe { Box::from_raw(data_ptr as *mut Data) };

    use Cmd::*;
    match msg {
        Msg::Cmd(client_index, cmd) => {
            let current_buffer =
                &mut global_data.buffers[global_data.clients[*client_index].buffer];
            let rope = &mut current_buffer.rope;
            let client = &global_data.clients[*client_index];
            match (cmd, &mut data.search_buffer) {
                (Cmd::ChangeMode(mode), ref mut search_buffer) => {
                    if *mode == Mode::Search {
                        **search_buffer = Some(SearchBuffer::default());
                    } else {
                        **search_buffer = None;
                    }
                }
                (InsertChar(c), Some(search_buffer)) => match global_data.clients[*client_index].mode {
                    Mode::Search => {
                        search_buffer.text.insert(search_buffer.index, *c);
                        send_cmd(*client_index, MoveCursor(Direction::Right, false));
                        if let Some(search_result) =
                            new_search_result(&search_buffer.text, rope)
                        {
                            data.search_results.insert(*client_index, search_result);
                        }
                    }
                    _ => {}
                },
                (DeleteChar(dir), Some(search_buffer)) => match global_data.clients[*client_index].mode {
                    Mode::Search => match dir {
                        DeleteDirection::Before => {
                            if search_buffer.index > 0 {
                                search_buffer.text.remove(search_buffer.index - 1);
                                send_cmd(*client_index, MoveCursor(Direction::Left, false));
                                if let Some(search_result) =
                                    new_search_result(&search_buffer.text, rope)
                                {
                                    data.search_results.insert(*client_index, search_result);
                                }
                            }
                        }
                        DeleteDirection::After => {}
                    },
                    _ => {}
                },
                (MoveCursor(dir, _selecting), Some(search_buffer)) => {
                    use Direction::*;
                    match global_data.clients[*client_index].mode {
                        Mode::Search => {
                            match dir {
                                Left => {
                                    if search_buffer.index > 0 {
                                        search_buffer.index -= 1;
                                    }
                                }
                                Right => {
                                    if search_buffer.index < search_buffer.text.len() {
                                        search_buffer.index += 1;
                                    }
                                }
                                _ => {} // Only left and right matter
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        _ => {}
    };
    std::mem::forget(data);
}

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
