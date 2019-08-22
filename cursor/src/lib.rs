use ropey::Rope;

use termion::cursor::{Goto, Show};
use types::{
    BackBuffer, Buffer, BufferIndex, Client, ClientIndex, Cmd, Color, DeleteDirection, Direction,
    GlobalData, JumpType, Mode, Msg, Point, Rect, SecondaryMap, Utils,
};

#[derive(Debug)]
struct Cursor {
    position: Point,
    stored_x: u16,
    selection_anchor: Option<Point>,
}

impl Default for Cursor {
    fn default() -> Cursor {
        let position = Point { x: 1, y: 0 };
        Cursor {
            selection_anchor: None,
            position,
            stored_x: 0,
        }
    }
}

fn get_ropey_index_from_cursor(position: &Point, rope: &Rope) -> usize {
    rope.line_to_char(position.y as usize) + position.x as usize - 1
}

fn get_char_range(
    position: &Point,
    selection_anchor: &Point,
    rope: &Rope,
) -> std::ops::RangeInclusive<usize> {
    let start_index = get_ropey_index_from_cursor(position, rope);
    let end_index = get_ropey_index_from_cursor(selection_anchor, rope);
    if start_index < end_index {
        start_index..=end_index
    } else {
        end_index..=start_index
    }
}

#[derive(Debug, Default)]
struct State {
    cursors: SecondaryMap<BufferIndex, Cursor>,
}

fn write_mode_status(back_buffer: &mut BackBuffer, client: &Client, utils: &Utils) {
    if let Some(Rect { w, h }) = client.size {
        let display = match client.mode {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Command => "COMMAND",
        };
        (utils.write_to_buffer)(
            back_buffer,
            &Point {
                x: w - display.len() as u16 - 1,
                y: h - 1,
            },
            display,
            None,
            None,
            None,
        );
    }
}

fn apply_selection_style(
    back_buffer: &mut BackBuffer,
    utils: &Utils,
    cursor: &Cursor,
    rope: &Rope,
) {
    if let Some(ref selection_anchor) = cursor.selection_anchor {
        let char_range = get_char_range(&cursor.position, selection_anchor, rope);
        let slice = rope.slice(char_range);
        let start_point = if *selection_anchor > cursor.position {
            Point {
                x: cursor.position.x + 3,
                y: cursor.position.y,
            }
        } else {
            Point {
                x: selection_anchor.x + 3,
                y: selection_anchor.y,
            }
        };
        (utils.style_rope_slice_range)(
            back_buffer,
            &slice,
            start_point,
            None,
            None,
            Some(Color {
                r: 0,
                g: 50,
                b: 200,
            }),
        );
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
    let mut data: Box<State> = unsafe { Box::from_raw(data_ptr as *mut State) };
    write_mode_status(back_buffer, &global_data.clients[*client], utils);
    use std::io::Write;
    let mut stream = global_data.clients[*client].stream.try_clone().unwrap();
    let cursor = get_or_insert_cursor(&mut data, &global_data, client);
    let buffer_index = global_data.clients[*client].buffer;
    let rope = &global_data.buffers[buffer_index].rope;
    apply_selection_style(back_buffer, utils, &cursor, rope);
    let current_buffer = global_data.clients[*client].buffer;
    if global_data.clients[*client].mode != Mode::Command {
        write!(
            stream,
            "{}{}",
            Show,
            Goto(
                cursor.position.x + 4, // +4 for line numbers
                cursor.position.y + 1 - global_data.buffers[current_buffer].start_line as u16
            )
        );
    }
    std::mem::forget(data);
}

fn get_new_x_position(cursor: &Cursor, rope: &Rope) -> u16 {
    let Cursor {
        position, stored_x, ..
    } = cursor;
    std::cmp::min(
        std::cmp::max(position.x, *stored_x),
        std::cmp::max(1, rope.line(position.y as usize).len_chars() as u16),
    )
}

fn get_or_insert_cursor<'a>(
    data: &'a mut Box<State>,
    global_data: &GlobalData,
    client: &ClientIndex,
) -> &'a mut Cursor {
    let buffer_index = global_data.clients[*client].buffer;
    if !data.cursors.contains_key(buffer_index) {
        data.cursors
            .insert(buffer_index, std::default::Default::default());
    }
    &mut data.cursors[buffer_index]
}

fn move_cursor_position(
    cursor: &mut Cursor,
    dir: &Direction,
    current_buffer: &mut Buffer,
    client: &Client,
) {
    let rope = &current_buffer.rope;
    use Direction::*;
    match dir {
        Left => {
            if cursor.position.x > 1 {
                cursor.position.x -= 1
            }
            cursor.stored_x = cursor.position.x;
        }
        Right => {
            cursor.position.x += 1;
            cursor.stored_x = cursor.position.x;
        }
        Up => {
            if cursor.position.y > 0 {
                cursor.position.y -= 1;
            }
            if (cursor.position.y as usize) < current_buffer.start_line {
                current_buffer.start_line -= 1;
            }
        }
        Down => {
            if (cursor.position.y as usize) + 2 < rope.len_lines() {
                cursor.position.y += 1;
            }
            if (cursor.position.y as usize)
                >= current_buffer.start_line
                    + (client.size.as_ref().map(|s| s.h).unwrap_or(0) as usize - 1)
            {
                current_buffer.start_line += 1;
            }
        }
    }
    // Make sure we don't venture to nowhere
    cursor.position.x = get_new_x_position(&cursor, &rope);
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
            let cursor = get_or_insert_cursor(&mut data, &global_data, client_index);
            let current_buffer =
                &mut global_data.buffers[global_data.clients[*client_index].buffer];
            let rope = &mut current_buffer.rope;
            let client = &global_data.clients[*client_index];
            match cmd {
                MoveCursor(dir, selecting) => {
                    if *selecting {
                        if cursor.selection_anchor.is_none() {
                            cursor.selection_anchor = Some(cursor.position.clone());
                        }
                    } else {
                        cursor.selection_anchor = None;
                    }
                    match global_data.clients[*client_index].mode {
                        Mode::Command => {}
                        _ => move_cursor_position(cursor, dir, current_buffer, client),
                    }
                }
                ChangeMode(ref mode) => {
                    global_data.clients[*client_index].mode = mode.clone();
                    cursor.selection_anchor = None;
                }
                InsertChar(c) => match global_data.clients[*client_index].mode {
                    Mode::Command => {}
                    _ => {
                        let index = get_ropey_index_from_cursor(&cursor.position, &rope);
                        rope.insert_char(index, *c);
                        if *c == '\n' {
                            send_cmd(*client_index, MoveCursor(Direction::Down, false));
                        } else {
                            send_cmd(*client_index, MoveCursor(Direction::Right, false));
                        }
                        send_cmd(*client_index, BufferModified);
                        cursor.selection_anchor = None;
                    }
                },
                DeleteChar(dir) => match global_data.clients[*client_index].mode {
                    Mode::Command => {}
                    _ => {
                        let index = get_ropey_index_from_cursor(&cursor.position, &rope);
                        match dir {
                            DeleteDirection::After => {
                                rope.remove(get_char_range(
                                    &cursor.position,
                                    cursor.selection_anchor.as_ref().unwrap_or(&cursor.position),
                                    &rope,
                                ));
                            }
                            DeleteDirection::Before => {
                                if cursor.position.x > 1 {
                                    rope.remove(index - 1..index);
                                    cursor.position.x -= 1
                                }
                            }
                        }
                        send_cmd(*client_index, BufferModified);
                        if let Some(selection_anchor) = cursor.selection_anchor.take() {
                            if selection_anchor < cursor.position {
                                cursor.position = selection_anchor.clone();
                            }
                        }
                        cursor.stored_x = cursor.position.x;
                    }
                },
                Jump(jump_type) => {
                    use JumpType::*;
                    match jump_type {
                        EndOfLine => {
                            let mut position = &mut cursor.position;
                            position.x = global_data.buffers
                                [global_data.clients[*client_index].buffer]
                                .rope
                                .line(position.y as usize)
                                .len_chars() as u16
                        }
                        StartOfLine => {
                            cursor.position.x = 1;
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
