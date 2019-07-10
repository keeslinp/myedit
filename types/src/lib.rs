use ropey::Rope;
use notify::DebouncedEvent;
use termion::event::Event;

#[derive(Debug)]
pub struct GlobalData {
    pub buffer: Option<Buffer>,
    pub mode: Mode,
}

#[derive(Debug)]
pub struct Buffer {
    pub rope: Rope,
    pub source: String,
}

#[derive(Debug)]
pub enum Mode {
  Normal,
  Insert,
}

#[derive(Debug)]
pub enum Msg {
    LoadFile(String),
    LibraryEvent(DebouncedEvent),
    StdinEvent(Event),
}
