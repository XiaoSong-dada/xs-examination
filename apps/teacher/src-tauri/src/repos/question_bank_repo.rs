use anyhow::{anyhow, Result};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryOrder,
    QueryFilter, Set,
};

use crate::models::question_bank_item::{
    ActiveModel, Column, Entity as QuestionBankItemEntity, Model as QuestionBankItemModel,
};
use crate::services::question_bank_service::QuestionBankWritePayload;

/// 查询全部全局题库题目并按最近更新时间倒序返回。
///
/// # 参数
/// - `db`: 数据库连接。
///
/// # 返回值
/// - 返回题库题目实体数组；数据库访问失败时返回错误。
pub async fn get_all_question_bank_items(
    db: &DatabaseConnection,
) -> Result<Vec<QuestionBankItemModel>> {
    let items = QuestionBankItemEntity::find()
        .order_by_desc(Column::UpdatedAt)
        .filter(Column::Content.is_not_null())
        .all(db)
        .await?;
    Ok(items)
}

/// 按 ID 查询单条全局题库题目。
///
/// # 参数
/// - `db`: 数据库连接。
/// - `id`: 题目 ID。
///
/// # 返回值
/// - 返回匹配的题目实体；若题目不存在或查询失败则返回错误。
pub async fn get_question_bank_item_by_id(
    db: &DatabaseConnection,
    id: &str,
) -> Result<QuestionBankItemModel> {
    let item = QuestionBankItemEntity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("题库题目不存在: {}", id))?;
    Ok(item)
}

/// 新增一条全局题库题目。
///
/// # 参数
/// - `db`: 数据库连接。
/// - `id`: 新题目 ID。
/// - `payload`: 题目写入载荷。
/// - `created_at`: 创建时间戳。
/// - `updated_at`: 更新时间戳。
///
/// # 返回值
/// - 返回新建后的题目实体；序列化或数据库写入失败时返回错误。
pub async fn insert_question_bank_item(
    db: &DatabaseConnection,
    id: String,
    payload: &QuestionBankWritePayload,
    created_at: i64,
    updated_at: i64,
) -> Result<QuestionBankItemModel> {
    let model = ActiveModel {
        id: Set(id),
        r#type: Set(payload.r#type.clone()),
        content: Set(payload.content.clone()),
        content_image_paths: Set(payload.serialized_content_image_paths()?),
        options: Set(payload.serialized_options()?),
        answer: Set(payload.answer.clone()),
        score: Set(payload.score),
        explanation: Set(payload.explanation.clone()),
        created_at: Set(created_at),
        updated_at: Set(updated_at),
    };

    let item = model.insert(db).await?;
    Ok(item)
}

/// 按 ID 更新一条全局题库题目。
///
/// # 参数
/// - `db`: 数据库连接。
/// - `id`: 待更新题目 ID。
/// - `payload`: 最新题目写入载荷。
/// - `updated_at`: 更新时间戳。
///
/// # 返回值
/// - 返回更新后的题目实体；若题目不存在、序列化失败或数据库写入失败则返回错误。
pub async fn update_question_bank_item_by_id(
    db: &DatabaseConnection,
    id: &str,
    payload: &QuestionBankWritePayload,
    updated_at: i64,
) -> Result<QuestionBankItemModel> {
    let existing = QuestionBankItemEntity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("题库题目不存在: {}", id))?;

    let mut model: ActiveModel = existing.into_active_model();
    model.r#type = Set(payload.r#type.clone());
    model.content = Set(payload.content.clone());
    model.content_image_paths = Set(payload.serialized_content_image_paths()?);
    model.options = Set(payload.serialized_options()?);
    model.answer = Set(payload.answer.clone());
    model.score = Set(payload.score);
    model.explanation = Set(payload.explanation.clone());
    model.updated_at = Set(updated_at);

    let item = model.update(db).await?;
    Ok(item)
}

/// 按 ID 删除一条全局题库题目。
///
/// # 参数
/// - `db`: 数据库连接。
/// - `id`: 待删除题目 ID。
///
/// # 返回值
/// - 删除成功返回 `()`；若题目不存在或数据库删除失败则返回错误。
pub async fn delete_question_bank_item_by_id(db: &DatabaseConnection, id: &str) -> Result<()> {
    let result = QuestionBankItemEntity::delete_by_id(id.to_string())
        .exec(db)
        .await?;
    if result.rows_affected == 0 {
        return Err(anyhow!("题库题目不存在: {}", id));
    }
    Ok(())
}