// 协议占位：将来放置与教师端/服务器交互的消息结构体定义

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatPayload {
    pub student_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerItem {
    pub question_id: String,
    pub answer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerSyncPayload {
    pub exam_id: String,
    pub student_id: String,
    pub answers: Vec<AnswerItem>,
}