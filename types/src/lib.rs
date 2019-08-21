use notify::DebouncedEvent;
use ropey::Rope;
pub use slotmap::{DefaultKey, KeyData, SecondaryMap, SlotMap};
use std::os::unix::net::UnixStream;

use termion::event::Event;

pub type ClientIndex = DefaultKey;

pub type BufferIndex = DefaultKey;

#[derive(Debug)]
pub struct GlobalData {
    pub buffer_keys: SlotMap<BufferIndex, ()>,
    pub buffers: SecondaryMap<BufferIndex, Buffer>,
    pub client_keys: SlotMap<ClientIndex, ()>,
    pub clients: SecondaryMap<ClientIndex, Client>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
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
    pub start_line: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum DeleteDirection {
    Before,
    After,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum JumpType {
    EndOfLine,
    StartOfLine,
    BeginningOfBuffer,
    EndOfBuffer,
    StartOfWord,
    EndOfWord,
    MatchingBrace,
}

use serde::{Deserialize, Serialize};
// use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RemoteCommand(pub ClientIndex, pub Cmd);

#[derive(Debug, Deserialize, Serialize)]
pub struct InitializeClient(pub ClientIndex);

#[derive(Debug, Deserialize, Serialize)]
pub enum Cmd {
    MoveSelection(Direction),
    MoveCursor(Direction),
    Quit,
    Kill,
    ChangeMode(Mode),
    InsertChar(char),
    DeleteChar(DeleteDirection),
    Jump(JumpType),
    RunCommand,
    WriteBuffer(std::path::PathBuf),
    LoadFile(std::path::PathBuf),
    BufferLoaded,
    BufferModified,
    SearchFiles,
    CleanRender,
    ResizeClient(Rect),
}

#[derive(Debug)]
pub struct Client {
    pub stream: UnixStream,
    pub buffer: DefaultKey,
    pub mode: Mode,
    pub back_buffer: BackBuffer,
    pub size: Option<Rect>, // We don't know right away
}

#[derive(Debug)]
pub enum Msg {
    LibraryEvent(DebouncedEvent),
    StdinEvent(ClientIndex, Event),
    Cmd(ClientIndex, Cmd),
    NewClient(UnixStream),
}

#[derive(Debug, Clone, Eq, PartialEq, Default, PartialOrd)]
pub struct Point {
    pub y: u16,
    pub x: u16,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Rect {
    pub w: u16,
    pub h: u16,
}

pub struct Utils {
    pub write_to_buffer:
        fn(&mut BackBuffer, &Point, &str, Option<Style>, Option<Color>, Option<Color>),
    pub style_range:
        fn(&mut BackBuffer, &Point, usize, Option<Style>, Option<Color>, Option<Color>),
    pub info: fn(&str),
    pub debug: fn(&str),
    pub warn: fn(&str),
}
