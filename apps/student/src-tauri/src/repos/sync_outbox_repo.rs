use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set};

use crate::db::entities::sync_outbox;

/// 获取待同步的答案记录。
pub async fn get_pending_answer_syncs(
    db: &DatabaseConnection,
    max_count: usize,
) -> Result<Vec<sync_outbox::Model>> {
    let rows = sync_outbox::Entity::find()
        .filter(sync_outbox::Column::EventType.eq("ANSWER_SYNC".to_string()))
        .filter(sync_outbox::Column::Status.is_in(["pending", "failed"]))
        .order_by_asc(sync_outbox::Column::CreatedAt)
        .limit(max_count as u64)
        .all(db)
        .await?;
    Ok(rows)
}

/// 标记同步记录为已发送。
pub async fn mark_outbox_sent(
    db: &DatabaseConnection,
    row: sync_outbox::Model,
    updated_at: i64,
) -> Result<()> {
    let mut model: sync_outbox::ActiveModel = row.into();
    model.status = Set("sent".to_string());
    model.updated_at = Set(updated_at);
    model.last_error = Set(None);
    model.update(db).await?;
    Ok(())
}

/// 标记同步记录为失败。
pub async fn mark_outbox_failed(
    db: &DatabaseConnection,
    row: sync_outbox::Model,
    error_message: &str,
    updated_at: i64,
) -> Result<()> {
    let next_retry_count = row.retry_count + 1;
    let mut model: sync_outbox::ActiveModel = row.into();
    model.status = Set("failed".to_string());
    model.retry_count = Set(next_retry_count);
    model.last_error = Set(Some(error_message.to_string()));
    model.updated_at = Set(updated_at);
    model.update(db).await?;
    Ok(())
}

/// 标记同步记录为已同步。
pub async fn mark_outbox_synced(
    db: &DatabaseConnection,
    session_id: &str,
    question_ids: &[String],
    updated_at: i64,
) -> Result<()> {
    let full_sync = question_ids.is_empty();

    let mut outbox_query = sync_outbox::Entity::find()
        .filter(sync_outbox::Column::SessionId.eq(session_id.to_string()))
        .filter(sync_outbox::Column::EventType.eq("ANSWER_SYNC".to_string()))
        .filter(sync_outbox::Column::Status.is_in(["pending", "failed", "sent"]));

    if !full_sync {
        let aggregate_ids: Vec<String> = question_ids
            .iter()
            .map(|qid| format!("{}:{}", session_id, qid))
            .collect();
        outbox_query = outbox_query.filter(
            sync_outbox::Column::AggregateId.is_in(aggregate_ids),
        );
    }

    let outbox_rows = outbox_query.all(db).await?;
    for row in outbox_rows {
        let mut model: sync_outbox::ActiveModel = row.into();
        model.status = Set("synced".to_string());
        model.updated_at = Set(updated_at);
        model.last_error = Set(None);
        model.update(db).await?;
    }

    Ok(())
}

/// 标记同步记录为失败（批量）。
pub async fn mark_outbox_failed_batch(
    db: &DatabaseConnection,
    session_id: &str,
    question_ids: &[String],
    error_message: &str,
    failed_at: i64,
) -> Result<()> {
    let full_sync = question_ids.is_empty();

    let mut outbox_query = sync_outbox::Entity::find()
        .filter(sync_outbox::Column::SessionId.eq(session_id.to_string()))
        .filter(sync_outbox::Column::EventType.eq("ANSWER_SYNC".to_string()))
        .filter(sync_outbox::Column::Status.is_in(["pending", "failed", "sent"]));

    if !full_sync {
        let aggregate_ids: Vec<String> = question_ids
            .iter()
            .map(|qid| format!("{}:{}", session_id, qid))
            .collect();
        outbox_query = outbox_query
            .filter(sync_outbox::Column::AggregateId.is_in(aggregate_ids));
    }

    let outbox_rows = outbox_query.all(db).await?;
    for row in outbox_rows {
        let next_retry_count = row.retry_count + 1;
        let mut model: sync_outbox::ActiveModel = row.into();
        model.status = Set("failed".to_string());
        model.retry_count = Set(next_retry_count);
        model.last_error = Set(Some(error_message.to_string()));
        model.updated_at = Set(failed_at);
        model.update(db).await?;
    }

    Ok(())
}