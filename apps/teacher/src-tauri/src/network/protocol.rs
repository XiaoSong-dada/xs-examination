use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageType {
    ExamStart,
    ExamPause,
    ExamEnd,
    ForceSubmit,
    Heartbeat,
    AnswerSync,
    Submit,
    StatusUpdate,
    CheatAlert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage<T> {
    pub r#type: MessageType,
    pub timestamp: i64,
    pub signature: String,
    pub payload: T,
}
