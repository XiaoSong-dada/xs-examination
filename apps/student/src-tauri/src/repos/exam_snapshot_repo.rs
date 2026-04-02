use anyhow::Result;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

use crate::db::entities::exam_snapshots;
use crate::schemas::control_protocol::DistributeExamPaperPayload;
use crate::schemas::exam_runtime_schema::ExamSnapshotDto;

/// 根据会话 ID 获取考试快照。
pub async fn get_snapshot_by_session_id(
    db: &DatabaseConnection,
    session_id: &str,
) -> Result<Option<exam_snapshots::Model>> {
    let snapshot = exam_snapshots::Entity::find_by_id(session_id.to_string())
        .one(db)
        .await?;
    Ok(snapshot)
}

/// 插入或更新考试快照。
pub async fn upsert_snapshot(
    db: &DatabaseConnection,
    session_id: &str,
    payload: &DistributeExamPaperPayload,
    ts: i64,
) -> Result<()> {
    let existing_snapshot = exam_snapshots::Entity::find_by_id(session_id.to_string())
        .one(db)
        .await?;

    match existing_snapshot {
        Some(row) => {
            let mut model: exam_snapshots::ActiveModel = row.into();
            model.exam_meta = Set(payload.exam_meta.clone().into_bytes());
            model.questions_payload = Set(payload.questions_payload.clone().into_bytes());
            model.downloaded_at = Set(payload.downloaded_at);
            model.expires_at = Set(payload.expires_at);
            model.package_status = Set(Some("legacy_ready".to_string()));
            model.package_path = Set(None);
            model.package_batch_id = Set(None);
            model.package_sha256 = Set(None);
            model.package_received_at = Set(None);
            model.assets_sync_status = Set(Some("pending".to_string()));
            model.assets_synced_at = Set(None);
            model.updated_at = Set(ts);
            model.update(db).await?;
        }
        None => {
            let model = exam_snapshots::ActiveModel {
                session_id: Set(session_id.to_string()),
                exam_meta: Set(payload.exam_meta.clone().into_bytes()),
                questions_payload: Set(payload.questions_payload.clone().into_bytes()),
                downloaded_at: Set(payload.downloaded_at),
                expires_at: Set(payload.expires_at),
                package_path: Set(None),
                package_status: Set(Some("legacy_ready".to_string())),
                package_batch_id: Set(None),
                package_sha256: Set(None),
                package_received_at: Set(None),
                assets_sync_status: Set(Some("pending".to_string())),
                assets_synced_at: Set(None),
                updated_at: Set(ts),
            };
            model.insert(db).await?;
        }
    }

    Ok(())
}

/// 将考试快照模型转换为 DTO。
pub fn snapshot_to_dto(snapshot: exam_snapshots::Model) -> ExamSnapshotDto {
    ExamSnapshotDto {
        session_id: snapshot.session_id,
        exam_meta: String::from_utf8_lossy(&snapshot.exam_meta).to_string(),
        questions_payload: String::from_utf8_lossy(&snapshot.questions_payload).to_string(),
        downloaded_at: snapshot.downloaded_at,
        expires_at: snapshot.expires_at,
        package_path: snapshot.package_path,
        package_status: snapshot.package_status,
        package_batch_id: snapshot.package_batch_id,
        package_sha256: snapshot.package_sha256,
        package_received_at: snapshot.package_received_at,
        assets_sync_status: snapshot.assets_sync_status,
        assets_synced_at: snapshot.assets_synced_at,
        updated_at: snapshot.updated_at,
    }
}