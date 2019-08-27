use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Search,
}
