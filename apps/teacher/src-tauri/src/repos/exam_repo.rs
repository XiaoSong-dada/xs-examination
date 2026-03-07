use anyhow::{anyhow, Result};
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryOrder, Set,
};

use crate::models::exam::{ActiveModel, Column, Entity as ExamEntity, Model as ExamModel};
use crate::services::exam_service::ExamWritePayload;

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

/// 根据考试 ID 查询考试详情。
///
/// # 参数
/// * `db` - SeaORM 数据库连接。
/// * `id` - 考试 UUID。
///
/// # 返回值
/// 返回考试详情；未找到考试时返回错误。
pub async fn get_exam_by_id(db: &DatabaseConnection, id: &str) -> Result<ExamModel> {
    let exam = ExamEntity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("考试不存在: {}", id))?;
    Ok(exam)
}

/// 插入一条考试记录。
///
/// # 参数
/// * `db` - SeaORM 数据库连接。
/// * `id` - 考试 UUID。
/// * `payload` - 考试写入参数。
/// * `now` - 当前毫秒时间戳。
///
/// # 返回值
/// 返回插入成功后的考试模型，插入失败时返回错误。
pub async fn insert_exam(
    db: &DatabaseConnection,
    id: String,
    payload: ExamWritePayload,
    now: i64,
) -> Result<ExamModel> {
    let model = ActiveModel {
        id: Set(id),
        title: Set(payload.title),
        description: Set(payload.description),
        start_time: Set(payload.start_time),
        end_time: Set(payload.end_time),
        pass_score: Set(payload.pass_score),
        status: Set(payload.status),
        shuffle_questions: Set(payload.shuffle_questions),
        shuffle_options: Set(payload.shuffle_options),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let exam = model.insert(db).await?;
    Ok(exam)
}

/// 根据考试 ID 更新一条考试记录。
///
/// # 参数
/// * `db` - SeaORM 数据库连接。
/// * `id` - 考试 UUID。
/// * `payload` - 需要更新的考试字段。
/// * `now` - 当前毫秒时间戳。
///
/// # 返回值
/// 返回更新成功后的考试模型；未找到考试或更新失败时返回错误。
pub async fn update_exam_by_id(
    db: &DatabaseConnection,
    id: &str,
    payload: ExamWritePayload,
    now: i64,
) -> Result<ExamModel> {
    let existing = ExamEntity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("考试不存在: {}", id))?;

    let mut model: ActiveModel = existing.into_active_model();
    model.title = Set(payload.title);
    model.description = Set(payload.description);
    model.start_time = Set(payload.start_time);
    model.end_time = Set(payload.end_time);
    model.pass_score = Set(payload.pass_score);
    model.status = Set(payload.status);
    model.shuffle_questions = Set(payload.shuffle_questions);
    model.shuffle_options = Set(payload.shuffle_options);
    model.updated_at = Set(now);

    let exam = model.update(db).await?;
    Ok(exam)
}

/// 根据考试 ID 删除考试记录。
///
/// # 参数
/// * `db` - SeaORM 数据库连接。
/// * `id` - 考试 UUID。
///
/// # 返回值
/// 删除成功返回 `()`；未找到考试或删除失败时返回错误。
pub async fn delete_exam_by_id(db: &DatabaseConnection, id: &str) -> Result<()> {
    let result = ExamEntity::delete_by_id(id.to_string()).exec(db).await?;
    if result.rows_affected == 0 {
        return Err(anyhow!("考试不存在: {}", id));
    }
    Ok(())
}
