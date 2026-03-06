use anyhow::Result;
use chrono::Utc;
use sea_orm::DatabaseConnection;

use crate::models::exam::Model as ExamModel;
use crate::repos::exam_repo;

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

/// 创建考试。
///
/// # 参数
/// * `db` - SeaORM 数据库连接。
/// * `title` - 考试标题。
///
/// # 返回值
/// 返回新创建的考试模型，插入失败时返回错误。
pub async fn create_exam(db: &DatabaseConnection, title: String) -> Result<ExamModel> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().timestamp_millis();
    exam_repo::insert_exam(db, id, title, now).await
}
