use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

/// Keybase is not running error
#[derive(Debug)]
pub struct KeybaseNotRunningError;

impl Error for KeybaseNotRunningError {}

impl fmt::Display for KeybaseNotRunningError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Keybase is not running")
    }
}

/// RPC request to the Grin wallet owner API.
#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub r#type: String,
    pub source: String,
    pub msg: Message,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub id: u64,
    pub conversation_id: String,
    pub channel: Channel,
    pub sender: Sender,
    pub sent_at: u64,
    pub sent_at_ms: u64,
    pub content: Content,
    pub prev: Option<String>,
    pub unread: bool,
    pub channel_mention: String,
    pub pagination: Pagination,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Channel {
    pub name: String,
    pub members_type: String,
    pub topic_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sender {
    pub uid: String,
    pub username: String,
    pub device_id: String,
    pub device_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    pub r#type: String,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pagination {
    pub next: String,
    pub previous: String,
    pub num: u32,
    pub last: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Text {
    pub body: String,
    pub payments: Option<u64>,
    pub user_mentions: Option<u64>,
    pub team_mentions: Option<u64>,
}
