use notify::DebouncedEvent;
use ropey::Rope;
use termion::event::Event;
use generational_arena::{Arena, Index};

#[derive(Debug)]
pub struct GlobalData {
    pub buffers: Arena<Buffer>,
    pub current_buffer: Index,
    pub mode: Mode,
    pub cursor: Cursor,
    pub command_buffer: CommandBuffer,
}

#[derive(Debug, Default)]
pub struct CommandBuffer {
    pub text: String,
    pub index: usize,
}

#[derive(Debug)]
pub struct Cursor {
    pub position: Point,
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

#[derive(Debug, Default)]
pub struct Buffer {
    pub rope: Rope,
    pub source: std::path::PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

#[derive(Debug)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug)]
pub enum DeleteDirection {
    Before,
    After,
}

#[derive(Debug)]
pub enum JumpType {
    EndOfLine,
    StartOfLine,
    BeginningOfBuffer,
    EndOfBuffer,
    StartOfWord,
    EndOfWord,
    MatchingBrace,
}

#[derive(Debug)]
pub enum Msg {
    LoadFile(std::path::PathBuf),
    WriteBuffer(std::path::PathBuf),
    LibraryEvent(DebouncedEvent),
    StdinEvent(Event),
    MoveCursor(Direction),
    Quit,
    ChangeMode(Mode),
    InsertChar(char),
    DeleteChar(DeleteDirection),
    Jump(JumpType),
    RunCommand,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Default)]
pub struct Rect {
    pub w: u16,
    pub h: u16,
}

pub struct Utils {
    pub write_to_buffer:
        fn(&mut BackBuffer, &Point, &str, Option<Style>, Option<Color>, Option<Color>),
}
