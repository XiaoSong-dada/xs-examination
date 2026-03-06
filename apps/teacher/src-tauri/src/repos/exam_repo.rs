use anyhow::Result;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryOrder, Set};

use crate::models::exam::{ActiveModel, Column, Entity as ExamEntity, Model as ExamModel};

/// 查询所有考试。
///
/// # 参数
/// * `db` - SeaORM 数据库连接。
///
/// # 返回值
/// 返回按 `created_at` 倒序排列的考试列表，查询失败时返回错误。
pub async fn get_all_exams(db: &DatabaseConnection) -> Result<Vec<ExamModel>> {
    let exams = ExamEntity::find()
        .order_by_desc(Column::CreatedAt)
        .all(db)
        .await?;
    Ok(exams)
}

/// 插入一条考试记录。
///
/// # 参数
/// * `db` - SeaORM 数据库连接。
/// * `id` - 考试 UUID。
/// * `title` - 考试标题。
/// * `now` - 当前毫秒时间戳。
///
/// # 返回值
/// 返回插入成功后的考试模型，插入失败时返回错误。
pub async fn insert_exam(
    db: &DatabaseConnection,
    id: String,
    title: String,
    now: i64,
) -> Result<ExamModel> {
    let model = ActiveModel {
        id: Set(id),
        title: Set(title),
        description: Set(None),
        start_time: Set(None),
        end_time: Set(None),
        pass_score: Set(0),
        status: Set("draft".to_string()),
        shuffle_questions: Set(0),
        shuffle_options: Set(0),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let exam = model.insert(db).await?;
    Ok(exam)
}
