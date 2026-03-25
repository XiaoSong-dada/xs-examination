use serde::{Deserialize, Serialize};

/// 考试元数据，对应 `exams` 表。
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Exam {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub pass_score: i64,
    /// draft | published | active | paused | finished
    pub status: String,
    pub shuffle_questions: i64,
    pub shuffle_options: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 题目，对应 `questions` 表。
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Question {
    pub id: String,
    pub exam_id: String,
    pub seq: i64,
    /// single | multi | judge | fill | essay
    #[sqlx(rename = "type")]
    pub question_type: String,
    pub content: String,
    /// JSON 数组字符串，客观题用，如 `[{"key":"A","text":"..."}]`
    pub options: Option<String>,
    pub answer: String,
    pub score: i64,
    pub explanation: Option<String>,
}

/// 考生信息，对应 `students` 表。
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Student {
    pub id: String,
    pub exam_id: String,
    pub student_no: String,
    pub name: String,
    pub ip_addr: Option<String>,
    /// waiting | active | submitted | offline | forced
    pub status: String,
    pub join_time: Option<i64>,
    pub submit_time: Option<i64>,
}

/// 学生答卷，对应 `answer_sheets` 表。
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AnswerSheet {
    pub id: String,
    pub student_exam_id: Option<String>,
    pub student_id: String,
    pub exam_id: String,
    pub question_id: String,
    pub answer: Option<String>,
    pub revision: Option<i64>,
    pub answer_updated_at: Option<i64>,
    pub received_at: Option<i64>,
    /// 0 = 错误，1 = 正确，NULL = 未评分（主观题）
    pub is_correct: Option<i64>,
    pub score: Option<i64>,
    pub synced_at: Option<i64>,
}

/// 成绩汇总，对应 `score_summary` 表。
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScoreSummary {
    pub id: String,
    pub exam_id: String,
    pub student_id: String,
    pub total_score: Option<i64>,
    pub is_passed: Option<i64>,
    pub graded_at: Option<i64>,
}

/// 防作弊告警日志，对应 `cheat_logs` 表。
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CheatLog {
    pub id: String,
    pub student_id: String,
    /// focus_lost | hotkey_detected | vm_detected | ...
    pub event_type: String,
    pub detail: Option<String>,
    pub occurred_at: i64,
}
