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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatPayload {
    #[serde(rename = "studentId")]
    pub student_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerItem {
    #[serde(rename = "questionId")]
    pub question_id: String,
    pub answer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerSyncPayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "studentId")]
    pub student_id: String,
    pub answers: Vec<AnswerItem>,
}
