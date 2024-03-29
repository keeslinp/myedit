use crate::{DeleteDirection, Direction, JumpType, Mode, Point, Rect};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Cmd {
    MoveCursor(Direction, bool),
    Quit,
    Kill,
    ChangeMode(Mode),
    InsertChar(char),
    InsertCharAtPoint(char, Point),
    InsertStringAtPoint(String, Point),
    DeleteCharRange(Point, Point),
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
    Yank,
    YankValue(String),
    Paste,
    PasteAtPoint(Point),
}
