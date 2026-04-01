use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, TransactionTrait};

use crate::db::entities::teacher_endpoints;
use crate::schemas::control_protocol::TeacherEndpointInput;

/// 替换所有教师端地址。
pub async fn replace_all_endpoints(
    db: &DatabaseConnection,
    endpoints: &[TeacherEndpointInput],
    ts: i64,
) -> Result<()> {
    let txn = db.begin().await?;

    teacher_endpoints::Entity::delete_many().exec(&txn).await?;

    for endpoint in endpoints {
        let model = teacher_endpoints::ActiveModel {
            id: Set(endpoint.id.clone()),
            endpoint: Set(endpoint.endpoint.clone()),
            name: Set(endpoint.name.clone()),
            remark: Set(endpoint.remark.clone()),
            is_master: Set(if endpoint.is_master { 1 } else { 0 }),
            last_seen: Set(None),
            created_at: Set(ts),
            updated_at: Set(ts),
        };
        model.insert(&txn).await?;
    }

    txn.commit().await?;
    Ok(())
}

/// 获取主教师端地址。
pub async fn get_master_endpoint(
    db: &DatabaseConnection,
) -> Result<Option<teacher_endpoints::Model>> {
    let master = teacher_endpoints::Entity::find()
        .filter(teacher_endpoints::Column::IsMaster.eq(1))
        .one(db)
        .await?;
    Ok(master)
}

/// 从输入列表中获取主教师端地址。
pub fn get_master_endpoint_from_input(
    endpoints: &[TeacherEndpointInput],
) -> Option<String> {
    endpoints
        .iter()
        .find(|e| e.is_master)
        .map(|e| e.endpoint.clone())
}