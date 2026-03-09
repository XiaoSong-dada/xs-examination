use anyhow::Result;
use sea_orm::{
    sea_query::{Alias, Expr, JoinType, Order, Query},
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    ExprTrait, Set, TransactionTrait,
};

use crate::models::student::Model as StudentModel;
use crate::models::student_exam::{
    ActiveModel, Column, Entity as StudentExamEntity, Model as StudentExamModel,
};

pub async fn get_student_exams_by_exam_id(
    db: &DatabaseConnection,
    exam_id: &str,
) -> Result<Vec<StudentExamModel>> {
    let rows = StudentExamEntity::find()
        .filter(Column::ExamId.eq(exam_id.to_string()))
        .all(db)
        .await?;

    Ok(rows)
}

pub async fn get_students_by_exam_id(
    db: &DatabaseConnection,
    exam_id: &str,
) -> Result<Vec<StudentModel>> {
    let se = Alias::new("se");
    let s = Alias::new("s");
    let query = Query::select()
        .column((s.clone(), Alias::new("id")))
        .column((s.clone(), Alias::new("student_no")))
        .column((s.clone(), Alias::new("name")))
        .column((s.clone(), Alias::new("created_at")))
        .column((s.clone(), Alias::new("updated_at")))
        .from_as(Alias::new("student_exams"), se.clone())
        .join_as(
            JoinType::LeftJoin,
            Alias::new("students"),
            s.clone(),
            Expr::col((s.clone(), Alias::new("id"))).equals((se.clone(), Alias::new("student_id"))),
        )
        .and_where(Expr::col((se.clone(), Alias::new("exam_id"))).eq(exam_id))
        .order_by((s.clone(), Alias::new("created_at")), Order::Desc)
        .to_owned();

    let rows = db.query_all(&query).await?;
    let mut students = Vec::with_capacity(rows.len());
    for row in rows {
        students.push(StudentModel {
            id: row.try_get("", "id")?,
            student_no: row.try_get("", "student_no")?,
            name: row.try_get("", "name")?,
            created_at: row.try_get("", "created_at")?,
            updated_at: row.try_get("", "updated_at")?,
        });
    }

    Ok(students)
}

pub async fn replace_students_by_exam_id(
    db: &DatabaseConnection,
    exam_id: &str,
    student_ids: Vec<String>,
) -> Result<Vec<StudentModel>> {
    let txn = db.begin().await?;

    StudentExamEntity::delete_many()
        .filter(Column::ExamId.eq(exam_id.to_string()))
        .exec(&txn)
        .await?;

    for student_id in student_ids {
        let model = ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            student_id: Set(student_id),
            exam_id: Set(exam_id.to_string()),
            ip_addr: Set(None),
            status: Set("waiting".to_string()),
            join_time: Set(None),
            submit_time: Set(None),
        };

        model.insert(&txn).await?;
    }

    txn.commit().await?;
    get_students_by_exam_id(db, exam_id).await
}
