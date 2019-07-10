use notify::DebouncedEvent;
use ropey::Rope;
use termion::event::Event;

#[derive(Debug)]
pub struct GlobalData {
    pub buffer: Option<Buffer>,
    pub mode: Mode,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Color {
    Red,
    Blue,
    Green,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Style {
    Underlined,
    Highlighted,
    Bold,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Cell {
    pub value: Option<char>,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub style: Option<Style>,
}

#[derive(Debug)]
pub struct BackBuffer {
    pub dim: Rect,
    pub cells: Vec<Cell>,
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug)]
pub struct Rect {
    pub w: u16,
    pub h: u16,
}

pub struct Utils {
    pub write_to_buffer: fn(&mut BackBuffer, &Point, &str, Option<Style>, Option<Color>, Option<Color>),
}
