use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageType {
    ExamStart,
    ExamPause,
    ExamEnd,
    ForceSubmit,
    Heartbeat,
    AnswerSync,
    AnswerSyncAck,
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

pub fn build_message<T>(message_type: MessageType, timestamp: i64, payload: T) -> WsMessage<T> {
    WsMessage {
        r#type: message_type,
        timestamp,
        signature: String::new(),
        payload,
    }
}

pub fn encode_message<T: Serialize>(message: &WsMessage<T>) -> anyhow::Result<String> {
    Ok(serde_json::to_string(message)?)
}

pub fn decode_value_message(text: &str) -> anyhow::Result<WsMessage<Value>> {
    Ok(serde_json::from_str(text)?)
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
    #[serde(default)]
    pub revision: Option<i64>,
    #[serde(rename = "answerUpdatedAt", default)]
    pub answer_updated_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerSyncPayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "sessionId", default)]
    pub session_id: Option<String>,
    #[serde(rename = "syncMode", default)]
    pub sync_mode: Option<String>,
    #[serde(rename = "batchId", default)]
    pub batch_id: Option<String>,
    pub answers: Vec<AnswerItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerSyncAckPayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "sessionId", default)]
    pub session_id: Option<String>,
    #[serde(rename = "syncMode", default)]
    pub sync_mode: Option<String>,
    #[serde(rename = "batchId", default)]
    pub batch_id: Option<String>,
    pub success: bool,
    pub message: String,
    #[serde(rename = "ackedAt")]
    pub acked_at: i64,
    #[serde(rename = "questionIds", default)]
    pub question_ids: Vec<String>,
    #[serde(rename = "failedQuestionIds", default)]
    pub failed_question_ids: Vec<String>,
    #[serde(rename = "successCount", default)]
    pub success_count: i64,
    #[serde(rename = "failedCount", default)]
    pub failed_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamStartPayload {
    #[serde(rename = "examId")]
    pub exam_id: String,
    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "startTime")]
    pub start_time: i64,
    #[serde(rename = "endTime")]
    pub end_time: Option<i64>,
    pub timestamp: i64,
}
