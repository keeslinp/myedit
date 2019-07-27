use ropey::Rope;
use std::sync::mpsc::Sender;
use termion::{
    cursor::{Goto, Show},
    event::{Event, Key},
    style, terminal_size,
};
use types::{
    BackBuffer, Cmd, DeleteDirection, Direction, GlobalData, JumpType, Mode, Msg, Point, Utils,
};

#[derive(Debug, Default)]
struct State {
    value: u8,
}

#[no_mangle]
pub fn render(
    global_data: &GlobalData,
    back_buffer: &mut BackBuffer,
    utils: &Utils,
    data_ptr: *mut c_void,
) {
    let data: Box<State> = unsafe { Box::from_raw(data_ptr as *mut State) };
    let (cols, rows) = terminal_size().unwrap();
    let display = match global_data.mode {
        Mode::Normal => "NORMAL",
        Mode::Insert => "INSERT",
        Mode::Command => "COMMAND",
    };
    (utils.write_to_buffer)(
        back_buffer,
        &Point {
            x: cols - display.len() as u16 - 1,
            y: rows - 1,
        },
        display,
        None,
        None,
        None,
    );
    if global_data.mode != Mode::Command {
        println!(
            "{}{}",
            Show,
            Goto(
                global_data.cursor.position.x,
                global_data.cursor.position.y + 1
                    - global_data.buffers[global_data.current_buffer].start_line as u16
            )
        );
    }
    std::mem::forget(data);
}

fn get_ropey_index_from_cursor(position: &Point, rope: &Rope) -> usize {
    rope.line_to_char(position.y as usize) + position.x as usize - 1
}

fn get_new_x_position(position: &Point, rope: &Rope) -> u16 {
    std::cmp::min(
        position.x,
        rope.line(position.y as usize).len_chars() as u16,
    )
}

#[no_mangle]
pub fn update(
    global_data: &mut GlobalData,
    cmd: &Msg,
    utils: &Utils,
    send_cmd: &Box<Fn(Cmd)>,
    data_ptr: *mut c_void,
) {
    let mut data: Box<State> = unsafe { Box::from_raw(data_ptr as *mut State) };
    let current_buffer = &mut global_data.buffers[global_data.current_buffer];
    let rope = &mut current_buffer.rope;
    use Cmd::*;
    match cmd {
        Msg::Cmd(cmd) => match cmd {
            MoveCursor(dir) => {
                use Direction::*;
                match global_data.mode {
                    Mode::Command => {}
                    _ => {
                        match dir {
                            Left => {
                                if global_data.cursor.position.x > 1 {
                                    global_data.cursor.position.x -= 1
                                }
                            }
                            Right => global_data.cursor.position.x += 1,
                            Up => {
                                if global_data.cursor.position.y > 1 {
                                    global_data.cursor.position.y -= 1;
                                }
                                if (global_data.cursor.position.y as usize)
                                    < current_buffer.start_line
                                {
                                    current_buffer.start_line -= 1;
                                }
                            }
                            Down => {
                                if (global_data.cursor.position.y as usize) + 1 < rope.len_lines() {
                                    global_data.cursor.position.y += 1;
                                }
                                let (_, rows) = terminal_size().expect("getting terminal size");
                                if (global_data.cursor.position.y as usize)
                                    >= current_buffer.start_line + (rows as usize - 1)
                                {
                                    current_buffer.start_line += 1;
                                }
                            }
                        }
                        // Make sure we don't venture to nowhere
                        global_data.cursor.position.x =
                            get_new_x_position(&global_data.cursor.position, &rope);
                    }
                }
            }
            ChangeMode(ref mode) => {
                global_data.mode = mode.clone();
            }
            InsertChar(c) => match global_data.mode {
                Mode::Command => {}
                _ => {
                    let index = get_ropey_index_from_cursor(&global_data.cursor.position, &rope);
                    rope.insert_char(index, *c);
                    if *c == '\n' {
                        send_cmd(MoveCursor(Direction::Down));
                    } else {
                        send_cmd(MoveCursor(Direction::Right));
                    }
                }
            },
            DeleteChar(dir) => match global_data.mode {
                Mode::Command => {}
                _ => {
                    let index = get_ropey_index_from_cursor(&global_data.cursor.position, &rope);
                    match dir {
                        DeleteDirection::After => {
                            rope.remove(index..index + 1);
                        }
                        DeleteDirection::Before => {
                            if global_data.cursor.position.x > 1 {
                                rope.remove(index - 1..index);
                                global_data.cursor.position.x -= 1
                            }
                        }
                    }
                }
            },
            Jump(jump_type) => {
                use JumpType::*;
                match jump_type {
                    EndOfLine => {
                        let mut position = &mut global_data.cursor.position;
                        position.x = global_data.buffers[global_data.current_buffer]
                            .rope
                            .line(position.y as usize)
                            .len_chars() as u16
                    }
                    StartOfLine => {
                        global_data.cursor.position.x = 1;
                    }
                    _ => {}
                }
            }
            _ => {}
        },
        _ => {}
    };
    std::mem::forget(data);
}

use std::ffi::c_void;

#[no_mangle]
pub fn init() -> *mut c_void {
    unsafe { Box::into_raw(Box::new(State::default())) as *mut c_void }
}

#[no_mangle]
pub fn cleanup(data: *mut c_void) {
    unsafe {
        let ptr = Box::from_raw(data);
        drop(ptr);
    }
}
