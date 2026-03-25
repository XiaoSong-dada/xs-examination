use anyhow::Result;
use chrono::Utc;
use sea_orm::DatabaseConnection;

use crate::models::exam::Model as ExamModel;
use crate::repos::exam_repo;

#[derive(Debug, Clone)]
pub struct ExamWritePayload {
    pub title: String,
    pub description: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub pass_score: i64,
    pub status: String,
    pub shuffle_questions: i64,
    pub shuffle_options: i64,
}

impl ExamWritePayload {
    pub fn with_defaults(
        title: String,
        description: Option<String>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        pass_score: Option<i64>,
        status: Option<String>,
        shuffle_questions: Option<i64>,
        shuffle_options: Option<i64>,
    ) -> Self {
        Self {
            title,
            description,
            start_time,
            end_time,
            pass_score: pass_score.unwrap_or(60),
            status: status.unwrap_or_else(|| "draft".to_string()),
            shuffle_questions: shuffle_questions.unwrap_or(0),
            shuffle_options: shuffle_options.unwrap_or(0),
        }
    }
}

/// 查询考试列表。
///
/// # 参数
/// * `db` - SeaORM 数据库连接。
///
/// # 返回值
/// 返回按创建时间倒序排列的考试模型列表，查询失败时返回错误。
pub async fn list_exams(db: &DatabaseConnection) -> Result<Vec<ExamModel>> {
    // 目前业务仅返回按创建时间倒序的所有考试，未来可加分页/权限校验等
    exam_repo::get_all_exams(db).await
}

/// 根据考试 ID 查询考试详情。
///
/// # 参数
/// * `db` - SeaORM 数据库连接。
/// * `id` - 考试 UUID。
///
/// # 返回值
/// 返回考试详情模型，查询失败时返回错误。
pub async fn get_exam_by_id(db: &DatabaseConnection, id: String) -> Result<ExamModel> {
    exam_repo::get_exam_by_id(db, &id).await
}

/// 创建考试。
///
/// # 参数
/// * `db` - SeaORM 数据库连接。
/// * `payload` - 考试写入参数。
///
/// # 返回值
/// 返回新创建的考试模型，插入失败时返回错误。
pub async fn create_exam(db: &DatabaseConnection, payload: ExamWritePayload) -> Result<ExamModel> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().timestamp_millis();
    exam_repo::insert_exam(db, id, payload, now).await
}

/// 根据考试 ID 更新考试。
///
/// # 参数
/// * `db` - SeaORM 数据库连接。
/// * `id` - 考试 UUID。
/// * `payload` - 考试写入参数。
///
/// # 返回值
/// 返回更新后的考试模型，更新失败时返回错误。
pub async fn update_exam(
    db: &DatabaseConnection,
    id: String,
    payload: ExamWritePayload,
) -> Result<ExamModel> {
    let now = Utc::now().timestamp_millis();
    exam_repo::update_exam_by_id(db, &id, payload, now).await
}

/// 根据考试 ID 删除考试。
///
/// # 参数
/// * `db` - SeaORM 数据库连接。
/// * `id` - 考试 UUID。
///
/// # 返回值
/// 删除成功返回 `()`，删除失败时返回错误。
pub async fn delete_exam(db: &DatabaseConnection, id: String) -> Result<()> {
    exam_repo::delete_exam_by_id(db, &id).await
}

pub async fn update_exam_status(
    db: &DatabaseConnection,
    id: String,
    status: String,
) -> Result<ExamModel> {
    let current = exam_repo::get_exam_by_id(db, &id).await?;
    let payload = ExamWritePayload {
        title: current.title,
        description: current.description,
        start_time: current.start_time,
        end_time: current.end_time,
        pass_score: current.pass_score,
        status,
        shuffle_questions: current.shuffle_questions,
        shuffle_options: current.shuffle_options,
    };

    let now = Utc::now().timestamp_millis();
    exam_repo::update_exam_by_id(db, &id, payload, now).await
}
