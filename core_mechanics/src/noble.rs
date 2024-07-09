use super::bank::Funds;

pub const NOBLE_VICTORY_POINTS: u8 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Noble {
    pub id: NobleId,
    pub cost: Funds,
}

impl Noble {
    pub fn new(id: NobleId, cost: Funds) -> Self {
        Self { id, cost }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NobleId {
    id: u8,
}

impl NobleId {
    pub fn new(id: u8) -> Self {
        Self { id }
    }
}
