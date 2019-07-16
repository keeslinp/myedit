use ropey::Rope;
use termion::{
    cursor::{Goto, Show},
    event::{Event, Key},
    style, terminal_size,
};
use types::{
    BackBuffer, DeleteDirection, Direction, GlobalData, JumpType, Mode, Msg, Point, Utils,
};

#[no_mangle]
pub fn render(_global_data: &GlobalData, _back_buffer: &mut BackBuffer, _utils: &Utils) {}

#[no_mangle]
pub fn update(global_data: &mut GlobalData, msg: &Msg, _utils: &Utils, send_msg: &Box<Fn(Msg)>) {
    use Msg::*;
    match msg {
        StdinEvent(evt) => {
            // Same for all modes
            match evt {
                Event::Key(k) => match k {
                    Key::Left => send_msg(Msg::MoveCursor(Direction::Left)),
                    Key::Right => send_msg(Msg::MoveCursor(Direction::Right)),
                    Key::Up => send_msg(Msg::MoveCursor(Direction::Up)),
                    Key::Down => send_msg(Msg::MoveCursor(Direction::Down)),
                    _ => {}
                },
                _ => {}
            }
            match global_data.mode {
                Mode::Normal => match evt {
                    Event::Key(Key::Char(c)) => match c {
                        'i' => send_msg(Msg::ChangeMode(Mode::Insert)),
                        'l' => send_msg(Msg::MoveCursor(Direction::Right)),
                        'h' => send_msg(Msg::MoveCursor(Direction::Left)),
                        'j' => send_msg(Msg::MoveCursor(Direction::Down)),
                        'k' => send_msg(Msg::MoveCursor(Direction::Up)),
                        'd' => send_msg(Msg::DeleteChar(DeleteDirection::After)),
                        'a' => {
                            send_msg(Msg::MoveCursor(Direction::Right));
                            send_msg(Msg::ChangeMode(Mode::Insert));
                        }
                        'A' => {
                            send_msg(Msg::Jump(JumpType::EndOfLine));
                            send_msg(Msg::ChangeMode(Mode::Insert));
                        }
                        'I' => {
                            send_msg(Msg::Jump(JumpType::StartOfLine));
                            send_msg(Msg::ChangeMode(Mode::Insert));
                        }
                        ':' => {
                            send_msg(Msg::ChangeMode(Mode::Command));
                        }
                        _ => {}
                    },
                    _ => {}
                },
                Mode::Insert => match evt {
                    Event::Key(k) => match k {
                        Key::Esc => send_msg(Msg::ChangeMode(Mode::Normal)),
                        Key::Backspace => send_msg(Msg::DeleteChar(DeleteDirection::Before)),
                        Key::Char(c) => send_msg(Msg::InsertChar(*c)),
                        _ => {}
                    },
                    _ => {}
                },
                Mode::Command => match evt {
                    Event::Key(key) => match key {
                        Key::Char('\n') => send_msg(Msg::RunCommand),
                        Key::Char(c) => send_msg(Msg::InsertChar(*c)),
                        Key::Backspace => send_msg(Msg::DeleteChar(DeleteDirection::Before)),
                        Key::Esc => send_msg(Msg::ChangeMode(Mode::Normal)),
                        _ => {}
                    },
                    _ => {}
                },
            }
        }
        _ => {}
    }
}

#[no_mangle]
pub fn init(global_data: &mut GlobalData) {}
