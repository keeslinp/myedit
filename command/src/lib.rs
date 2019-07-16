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
    let status_row_y = rows - 1;
    if global_data.mode == Mode::Command {
        (utils.write_to_buffer)(
            back_buffer,
            &Point { x: 0, y: rows - 1 },
            &format!(":{}", global_data.command_buffer.text),
            None,
            None,
            None,
        );
        println!(
            "{}{}",
            Show,
            Goto(global_data.command_buffer.index as u16 + 2, status_row_y)
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
pub fn update(global_data: &mut GlobalData, msg: &Msg, utils: &Utils, send_msg: &Box<Fn(Cmd)>) {
    use Cmd::*;
    match msg {
        Msg::Cmd(cmd) => match cmd {
            Cmd::RunCommand => {
                let mut command_words = global_data.command_buffer.text.split(" ");
                match command_words.next() {
                    Some("w") => {
                        let path = command_words
                            .next()
                            .map(|file_path| std::path::PathBuf::from(file_path))
                            .unwrap_or(
                                global_data.buffers[global_data.current_buffer]
                                    .source
                                    .clone(),
                            );
                        send_msg(Cmd::WriteBuffer(path));
                    }
                    Some("e") => {
                        let path = command_words
                            .next()
                            .map(|file_path| std::path::PathBuf::from(file_path))
                            .unwrap_or(
                                global_data.buffers[global_data.current_buffer]
                                    .source
                                    .clone(),
                            );
                        send_msg(Cmd::LoadFile(path));
                    }
                    Some("q") => send_msg(Cmd::Quit),
                    Some("wq") => {
                        send_msg(Cmd::WriteBuffer(
                            global_data.buffers[global_data.current_buffer]
                                .source
                                .clone(),
                        ));
                        send_msg(Cmd::Quit);
                    }
                    _ => {
                        // Unknown command
                    }
                }
                send_msg(Cmd::ChangeMode(Mode::Normal));
            }
            Cmd::ChangeMode(mode) => {
                if *mode == Mode::Command {
                    global_data.command_buffer.text = "".into();
                    global_data.command_buffer.index = 0;
                }
            }
            _ => {}
        },
        _ => {}
    };
}

#[no_mangle]
pub fn init(global_data: &mut GlobalData) {}
