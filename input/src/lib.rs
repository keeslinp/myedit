use ropey::Rope;
use types::{GlobalData, Msg, Mode, Utils, BackBuffer, Point};
use termion::{terminal_size, cursor::Goto, style, event::{Event, Key }};

#[no_mangle]
pub fn render(global_data: &GlobalData, back_buffer: &mut BackBuffer, utils: &Utils) {
    let (cols, rows) = terminal_size().unwrap();
    let display = match global_data.mode {
        Mode::Normal => "NORMAL",
        Mode::Insert => "INSERT",
    };
    (utils.write_to_buffer)(back_buffer, &Point { x: cols - display.len() as u16 - 1, y: rows - 1 }, display, None, None, None);
    // print!("{}{} {:?} {}", style::Invert, Goto(cols - 10, rows), global_data.mode, style::NoInvert);
}

#[no_mangle]
pub fn update(global_data: &mut GlobalData, msg: &Msg) {
    use Msg::*;
    match msg {
        StdinEvent(evt) => {
            match global_data.mode {
                Mode::Normal => {
                    match evt {
                        Event::Key(Key::Char(c)) => {
                            match c {
                                'i' => {
                                    global_data.mode = Mode::Insert;
                                },
                                'q' => {
                                    std::process::exit(0);
                                },
                                _ => {},
                            }
                        },
                        _ => {},
                    }
                },
                Mode::Insert => {
                    match evt {
                        Event::Key(k) => {
                            match k {
                                Key::Esc => {
                                    global_data.mode = Mode::Normal;
                                },
                                _ => {},
                            }
                        },
                        _ => {},
                    }
                },
            }
        },
        _ => {},
    }
}

#[no_mangle]
pub fn init(global_data: &mut GlobalData) {
}
