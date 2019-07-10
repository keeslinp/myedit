use termion::{color, style, cursor};
use types::{GlobalData, Msg};
use std::io::{stdout, Write};


#[no_mangle]
pub fn render(global_data: &GlobalData) {
    let stdout = std::io::stdout();
    if let Some(ref buffer) = global_data.buffer {
        let mut lock = stdout.lock();
        write!(lock, "{}", cursor::Hide);
        for (index, line) in buffer.rope.lines().enumerate() {
            write!(lock, "{}{}{}", termion::cursor::Goto(1, index as u16), termion::clear::CurrentLine, line.as_str().unwrap_or(""));
        }
        lock.flush().unwrap();
    }
}

#[no_mangle]
pub fn update(global_data: &GlobalData, msg: &Msg) {}

#[no_mangle]
pub fn init(global_data: &mut GlobalData) {
}
