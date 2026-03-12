use anyhow::{anyhow, Result};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter, QueryOrder, Set, TransactionTrait,
};

use crate::models::device::{ActiveModel, Column, Entity as DeviceEntity, Model as DeviceModel};
use crate::services::device_service::DeviceWritePayload;

pub async fn get_all_devices(
    db: &DatabaseConnection,
    ip: Option<&str>,
    name: Option<&str>,
) -> Result<Vec<DeviceModel>> {
    let mut cond = Condition::all();

    if let Some(ip_keyword) = ip.filter(|value| !value.trim().is_empty()) {
        cond = cond.add(Column::Ip.contains(ip_keyword.trim()));
    }

    if let Some(name_keyword) = name.filter(|value| !value.trim().is_empty()) {
        cond = cond.add(Column::Name.contains(name_keyword.trim()));
    }

    let devices = DeviceEntity::find()
        .filter(cond)
        .order_by_desc(Column::Id)
        .all(db)
        .await?;

    Ok(devices)
}

pub async fn get_device_by_id(db: &DatabaseConnection, id: &str) -> Result<DeviceModel> {
    let device = DeviceEntity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("设备不存在: {}", id))?;
    Ok(device)
}

pub async fn insert_device(
    db: &DatabaseConnection,
    id: String,
    payload: DeviceWritePayload,
) -> Result<DeviceModel> {
    let duplicated = DeviceEntity::find()
        .filter(
            Condition::all()
                .add(Column::Ip.eq(payload.ip.clone()))
                .add(Column::Name.eq(payload.name.clone())),
        )
        .one(db)
        .await?;

    if duplicated.is_some() {
        return Err(anyhow!("设备已存在: {} / {}", payload.ip, payload.name));
    }

    let model = ActiveModel {
        id: Set(id),
        ip: Set(payload.ip),
        name: Set(payload.name),
    };

    let device = model.insert(db).await?;
    Ok(device)
}

pub async fn update_device_by_id(
    db: &DatabaseConnection,
    id: &str,
    payload: DeviceWritePayload,
) -> Result<DeviceModel> {
    let duplicated = DeviceEntity::find()
        .filter(
            Condition::all()
                .add(Column::Ip.eq(payload.ip.clone()))
                .add(Column::Name.eq(payload.name.clone()))
                .add(Column::Id.ne(id.to_string())),
        )
        .one(db)
        .await?;

    if duplicated.is_some() {
        return Err(anyhow!("设备已存在: {} / {}", payload.ip, payload.name));
    }

    let existing = DeviceEntity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("设备不存在: {}", id))?;

    let mut model: ActiveModel = existing.into_active_model();
    model.ip = Set(payload.ip);
    model.name = Set(payload.name);

    let device = model.update(db).await?;
    Ok(device)
}

pub async fn delete_device_by_id(db: &DatabaseConnection, id: &str) -> Result<()> {
    let result = DeviceEntity::delete_by_id(id.to_string()).exec(db).await?;
    if result.rows_affected == 0 {
        return Err(anyhow!("设备不存在: {}", id));
    }
    Ok(())
}

pub async fn replace_devices_by_ips(db: &DatabaseConnection, ips: Vec<String>) -> Result<Vec<DeviceModel>> {
    let txn = db.begin().await?;

    DeviceEntity::delete_many().exec(&txn).await?;

    let mut inserted = Vec::with_capacity(ips.len());
    for (index, ip) in ips.into_iter().enumerate() {
        let model = ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            ip: Set(ip),
            name: Set(format!("学生设备{}", index + 1)),
        };

        inserted.push(model.insert(&txn).await?);
    }

    txn.commit().await?;
    Ok(inserted)
}
