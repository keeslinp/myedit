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

#[no_mangle]
pub fn render(global_data: &GlobalData, back_buffer: &mut BackBuffer, utils: &Utils) {
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
            Goto(global_data.cursor.position.x, global_data.cursor.position.y)
        );
    }
    // print!("{}{} {:?} {}", style::Invert, Goto(cols - 10, rows), global_data.mode, style::NoInvert);
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
pub fn update(global_data: &mut GlobalData, cmd: &Msg, utils: &Utils, send_cmd: &Box<Fn(Cmd)>) {
    let mut rope = &mut global_data.buffers[global_data.current_buffer].rope;
    use Cmd::*;
    match cmd {
        Msg::Cmd(cmd) => match cmd {
            MoveCursor(dir) => {
                use Direction::*;
                match global_data.mode {
                    Mode::Command => {
                        match dir {
                            Left => {
                                if global_data.command_buffer.index > 0 {
                                    global_data.command_buffer.index -= 1;
                                }
                            }
                            Right => {
                                if global_data.command_buffer.index
                                    < global_data.command_buffer.text.len()
                                {
                                    global_data.command_buffer.index += 1;
                                }
                            }
                            _ => {} // Only left and right matter
                        }
                    }
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
                            }
                            Down => {
                                global_data.cursor.position.y += 1;
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
                Mode::Command => {
                    global_data
                        .command_buffer
                        .text
                        .insert(global_data.command_buffer.index, *c);
                    send_cmd(MoveCursor(Direction::Right));
                }
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
                Mode::Command => match dir {
                    DeleteDirection::Before => {
                        global_data
                            .command_buffer
                            .text
                            .remove(global_data.command_buffer.index - 1);
                        send_cmd(MoveCursor(Direction::Left));
                    }
                    DeleteDirection::After => {}
                },
                _ => {
                    let index = get_ropey_index_from_cursor(&global_data.cursor.position, &rope);
                    match dir {
                        DeleteDirection::After => {
                            rope.remove(index..index + 1);
                        }
                        DeleteDirection::Before => {
                            if (global_data.cursor.position.x > 1) {
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
}

#[no_mangle]
pub fn init(global_data: &mut GlobalData) {}
