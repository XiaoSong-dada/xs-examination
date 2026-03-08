use anyhow::{anyhow, Result};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter, QueryOrder, Set, TransactionTrait,
};
use std::collections::HashSet;

use crate::models::student::{ActiveModel, Column, Entity as StudentEntity, Model as StudentModel};
use crate::services::student_service::StudentWritePayload;

#[derive(Debug, Clone)]
pub struct StudentBatchInsertItem {
    pub id: String,
    pub student_no: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

pub async fn get_all_students(db: &DatabaseConnection) -> Result<Vec<StudentModel>> {
    let students = StudentEntity::find()
        .order_by_desc(Column::CreatedAt)
        .all(db)
        .await?;
    Ok(students)
}

pub async fn get_student_by_id(db: &DatabaseConnection, id: &str) -> Result<StudentModel> {
    let student = StudentEntity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("学生不存在: {}", id))?;
    Ok(student)
}

pub async fn insert_student(
    db: &DatabaseConnection,
    id: String,
    payload: StudentWritePayload,
    created_at: i64,
    updated_at: i64,
) -> Result<StudentModel> {
    let duplicated = StudentEntity::find()
        .filter(Column::StudentNo.eq(payload.student_no.clone()))
        .one(db)
        .await?;
    if duplicated.is_some() {
        return Err(anyhow!("学号已存在: {}", payload.student_no));
    }

    let model = ActiveModel {
        id: Set(id),
        student_no: Set(payload.student_no),
        name: Set(payload.name),
        created_at: Set(created_at),
        updated_at: Set(updated_at),
    };

    let student = model.insert(db).await?;
    Ok(student)
}

pub async fn update_student_by_id(
    db: &DatabaseConnection,
    id: &str,
    payload: StudentWritePayload,
    now: i64,
) -> Result<StudentModel> {
    let duplicated = StudentEntity::find()
        .filter(
            Condition::all()
                .add(Column::StudentNo.eq(payload.student_no.clone()))
                .add(Column::Id.ne(id.to_string())),
        )
        .one(db)
        .await?;
    if duplicated.is_some() {
        return Err(anyhow!("学号已存在: {}", payload.student_no));
    }

    let existing = StudentEntity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("学生不存在: {}", id))?;

    let mut model: ActiveModel = existing.into_active_model();
    model.student_no = Set(payload.student_no);
    model.name = Set(payload.name);
    model.updated_at = Set(now);

    let student = model.update(db).await?;
    Ok(student)
}

pub async fn delete_student_by_id(db: &DatabaseConnection, id: &str) -> Result<()> {
    let result = StudentEntity::delete_by_id(id.to_string()).exec(db).await?;
    if result.rows_affected == 0 {
        return Err(anyhow!("学生不存在: {}", id));
    }
    Ok(())
}

pub async fn insert_students_batch(
    db: &DatabaseConnection,
    rows: Vec<StudentBatchInsertItem>,
) -> Result<Vec<StudentModel>> {
    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let mut seen = HashSet::new();
    for row in &rows {
        if !seen.insert(row.student_no.clone()) {
            return Err(anyhow!("导入数据中存在重复学号: {}", row.student_no));
        }
    }

    let student_nos = rows
        .iter()
        .map(|row| row.student_no.clone())
        .collect::<Vec<_>>();

    let duplicated = StudentEntity::find()
        .filter(Column::StudentNo.is_in(student_nos))
        .one(db)
        .await?;
    if let Some(exists) = duplicated {
        return Err(anyhow!("学号已存在: {}", exists.student_no));
    }

    let txn = db.begin().await?;
    let mut inserted = Vec::with_capacity(rows.len());

    for row in rows {
        let model = ActiveModel {
            id: Set(row.id),
            student_no: Set(row.student_no),
            name: Set(row.name),
            created_at: Set(row.created_at),
            updated_at: Set(row.updated_at),
        };
        inserted.push(model.insert(&txn).await?);
    }

    txn.commit().await?;
    Ok(inserted)
}
