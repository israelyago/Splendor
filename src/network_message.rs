use anyhow::Result;
use core_mechanics::board::{Action, Board};
use iroh::NodeId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Message {
    JoinTable {
        from: NodeId,
        message_id: Uuid,
    },
    StartGame {
        from: NodeId,
        // board_state: Board,
        message_id: Uuid,
    },
    Action {
        from: NodeId,
        action: Action,
        message_id: Uuid,
    },
    Announcement {
        from: NodeId,
        message: String,
        message_id: Uuid,
    },
    BoardStateUpdated {
        from: NodeId,
        board: Board,
        message_id: Uuid,
    },
}

impl Message {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(Into::into)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("serde_json::to_vec is infallible")
    }
}
