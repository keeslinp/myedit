use std::ffi::c_void;
use termion::event::{Event, Key};
use types::{
    BackBuffer, ClientIndex, Cmd, DeleteDirection, Direction, GlobalData, JumpType, Mode, Msg,
    Utils,
};

#[derive(Debug, Default)]
struct Data {
    value: u8,
}

#[no_mangle]
pub fn render(_global_data: &GlobalData, _back_buffer: &mut BackBuffer, _utils: &Utils) {}

#[no_mangle]
pub fn update(
    global_data: &mut GlobalData,
    msg: &Msg,
    _utils: &Utils,
    send_cmd: &Box<Fn(ClientIndex, Cmd)>,
) {
    match msg {
        Msg::StdinEvent(client, evt) => {
            // Same for all modes
            match evt {
                Event::Key(k) => match k {
                    Key::Left => send_cmd(*client, Cmd::MoveCursor(Direction::Left, false)),
                    Key::Right => send_cmd(*client, Cmd::MoveCursor(Direction::Right, false)),
                    Key::Up => send_cmd(*client, Cmd::MoveCursor(Direction::Up, false)),
                    Key::Down => send_cmd(*client, Cmd::MoveCursor(Direction::Down, false)),
                    _ => {}
                },
                _ => {}
            }
            match global_data.clients[*client].mode {
                Mode::Normal => match evt {
                    Event::Key(Key::Char(c)) => match c {
                        'i' => send_cmd(*client, Cmd::ChangeMode(Mode::Insert)),
                        'l' => send_cmd(*client, Cmd::MoveCursor(Direction::Right, false)),
                        'h' => send_cmd(*client, Cmd::MoveCursor(Direction::Left, false)),
                        'j' => send_cmd(*client, Cmd::MoveCursor(Direction::Down, false)),
                        'k' => send_cmd(*client, Cmd::MoveCursor(Direction::Up, false)),
                        'L' => send_cmd(*client, Cmd::MoveCursor(Direction::Right, true)),
                        'H' => send_cmd(*client, Cmd::MoveCursor(Direction::Left, true)),
                        'J' => send_cmd(*client, Cmd::MoveCursor(Direction::Down, true)),
                        'K' => send_cmd(*client, Cmd::MoveCursor(Direction::Up, true)),
                        'd' => send_cmd(*client, Cmd::DeleteChar(DeleteDirection::After)),
                        'a' => {
                            send_cmd(*client, Cmd::MoveCursor(Direction::Right, false));
                            send_cmd(*client, Cmd::ChangeMode(Mode::Insert));
                        }
                        'A' => {
                            send_cmd(*client, Cmd::Jump(JumpType::EndOfLine));
                            send_cmd(*client, Cmd::ChangeMode(Mode::Insert));
                        }
                        'I' => {
                            send_cmd(*client, Cmd::Jump(JumpType::StartOfLine));
                            send_cmd(*client, Cmd::ChangeMode(Mode::Insert));
                        }
                        ':' => {
                            send_cmd(*client, Cmd::ChangeMode(Mode::Command));
                        }
                        _ => {}
                    },
                    Event::Key(Key::Ctrl(c)) => match c {
                        'p' => send_cmd(*client, Cmd::SearchFiles),
                        _ => {}
                    },
                    _ => {}
                },
                Mode::Insert => match evt {
                    Event::Key(k) => match k {
                        Key::Esc => send_cmd(*client, Cmd::ChangeMode(Mode::Normal)),
                        Key::Backspace => {
                            send_cmd(*client, Cmd::DeleteChar(DeleteDirection::Before))
                        }
                        Key::Char(c) => send_cmd(*client, Cmd::InsertChar(*c)),
                        _ => {}
                    },
                    _ => {}
                },
                Mode::Command => match evt {
                    Event::Key(key) => match key {
                        Key::Char('\n') => send_cmd(*client, Cmd::RunCommand),
                        Key::Char(c) => send_cmd(*client, Cmd::InsertChar(*c)),
                        Key::Backspace => {
                            send_cmd(*client, Cmd::DeleteChar(DeleteDirection::Before))
                        }
                        Key::Esc => send_cmd(*client, Cmd::ChangeMode(Mode::Normal)),
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
pub fn init() -> *mut c_void {
    Box::into_raw(Box::new(Data::default())) as *mut c_void
}

#[no_mangle]
pub fn cleanup(data: *mut c_void) {
    unsafe {
        let ptr = Box::from_raw(data as *mut Data);
        drop(ptr);
    }
}
