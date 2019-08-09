use ropey::Rope;
use std::ffi::c_void;
use std::sync::mpsc::Sender;
use termion::{
    cursor::{Goto, Show},
    event::{Event, Key},
    style, terminal_size,
};
use types::{
    BackBuffer, ClientIndex, Cmd, DeleteDirection, Direction, GlobalData, JumpType, Mode, Msg,
    Point, Utils,
};

#[derive(Debug, Default)]
struct CommandBuffer {
    pub text: String,
    pub index: usize,
}

#[derive(Debug, Default)]
struct Data {
    command_buffer: CommandBuffer,
}

#[no_mangle]
pub fn render(
    global_data: &GlobalData,
    client: ClientIndex,
    back_buffer: &mut BackBuffer,
    utils: &Utils,
    data_ptr: *mut c_void,
) {
    let mode = &global_data.clients[client].mode;
    let data = unsafe { Box::from_raw(data_ptr as *mut Data) };
    let (cols, rows) = terminal_size().unwrap();
    let status_row_y = rows - 1;
    if *mode == Mode::Command {
        (utils.write_to_buffer)(
            back_buffer,
            &Point { x: 0, y: rows - 1 },
            &format!(":{}", data.command_buffer.text),
            None,
            None,
            None,
        );
        println!(
            "{}{}",
            Show,
            Goto(data.command_buffer.index as u16 + 2, status_row_y)
        );
    }
    // print!("{}{} {:?} {}", style::Invert, Goto(cols - 10, rows), global_data.mode, style::NoInvert);
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
    msg: &Msg,
    utils: &Utils,
    send_cmd: &Box<Fn(Cmd)>,
    data_ptr: *mut c_void,
) {
    let mut data = unsafe { Box::from_raw(data_ptr as *mut Data) };
    use Cmd::*;
    match msg {
        Msg::Cmd(client, cmd) => match cmd {
            Cmd::RunCommand => {
                let mut command_words = data.command_buffer.text.split(" ");
                match command_words.next() {
                    Some("w") => {
                        let path = command_words
                            .next()
                            .map(|file_path| std::path::PathBuf::from(file_path))
                            .unwrap_or(
                                global_data.buffers[global_data.clients[*client].buffer]
                                    .source
                                    .clone(),
                            );
                        send_cmd(Cmd::WriteBuffer(path));
                    }
                    Some("e") => {
                        let path = command_words
                            .next()
                            .map(|file_path| std::path::PathBuf::from(file_path))
                            .unwrap_or(
                                global_data.buffers[global_data.clients[*client].buffer]
                                    .source
                                    .clone(),
                            );
                        send_cmd(Cmd::LoadFile(path));
                    }
                    Some("q") => send_cmd(Cmd::Quit),
                    Some("wq") => {
                        send_cmd(Cmd::WriteBuffer(
                            global_data.buffers[global_data.clients[*client].buffer]
                                .source
                                .clone(),
                        ));
                        send_cmd(Cmd::Quit);
                    }
                    _ => {
                        // Unknown command
                    }
                }
                send_cmd(Cmd::ChangeMode(Mode::Normal));
            }
            Cmd::ChangeMode(mode) => {
                if *mode == Mode::Command {
                    data.command_buffer.text = "".into();
                    data.command_buffer.index = 0;
                }
            }
            InsertChar(c) => match global_data.clients[*client].mode {
                Mode::Command => {
                    data.command_buffer
                        .text
                        .insert(data.command_buffer.index, *c);
                    send_cmd(MoveCursor(Direction::Right));
                }
                _ => {}
            },
            DeleteChar(dir) => match global_data.clients[*client].mode {
                Mode::Command => match dir {
                    DeleteDirection::Before => {
                        data.command_buffer
                            .text
                            .remove(data.command_buffer.index - 1);
                        send_cmd(MoveCursor(Direction::Left));
                    }
                    DeleteDirection::After => {}
                },
                _ => {}
            },
            MoveCursor(dir) => {
                use Direction::*;
                match global_data.clients[*client].mode {
                    Mode::Command => {
                        match dir {
                            Left => {
                                if data.command_buffer.index > 0 {
                                    data.command_buffer.index -= 1;
                                }
                            }
                            Right => {
                                if data.command_buffer.index < data.command_buffer.text.len() {
                                    data.command_buffer.index += 1;
                                }
                            }
                            _ => {} // Only left and right matter
                        }
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
