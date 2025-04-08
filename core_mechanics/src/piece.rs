use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Piece {
    Red,
    Green,
    Blue,
    Brown,
    White,
    Golden,
}
