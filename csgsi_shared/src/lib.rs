use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

/// Message that is sent from the backend to the frontend.
/// Can be either a GSI state object, or a log message.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Message {
    State(Box<RawValue>),
    Log(String),
}

impl Message {
    pub fn from_state_payload(payload: String) -> Result<Message, serde_json::Error> {
        Ok(Message::State(RawValue::from_string(payload)?))
    }

    pub fn from_log(line: String) -> Message {
        Message::Log(line)
    }
}
