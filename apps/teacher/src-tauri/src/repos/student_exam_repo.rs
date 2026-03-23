use anyhow::Result;
use sea_orm::{
    sea_query::{Alias, Expr, JoinType, Order, Query},
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    ExprTrait, Set, TransactionTrait,
};
use std::collections::HashMap;

use crate::models::student::Model as StudentModel;
use crate::models::student_exam::{
    ActiveModel, Column, Entity as StudentExamEntity, Model as StudentExamModel,
};
use crate::schemas::student_exam_schema;

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

pub async fn get_student_device_assignments_by_exam_id(
    db: &DatabaseConnection,
    exam_id: &str,
) -> Result<Vec<student_exam_schema::StudentDeviceAssignDto>> {
    let se = Alias::new("se");
    let s = Alias::new("s");
    let d = Alias::new("d");

    let query = Query::select()
        .expr_as(
            Expr::col((se.clone(), Alias::new("id"))),
            Alias::new("student_exam_id"),
        )
        .column((se.clone(), Alias::new("student_id")))
        .column((s.clone(), Alias::new("student_no")))
        .expr_as(
            Expr::col((s.clone(), Alias::new("name"))),
            Alias::new("student_name"),
        )
        .column((se.clone(), Alias::new("ip_addr")))
        .expr_as(
            Expr::col((d.clone(), Alias::new("name"))),
            Alias::new("device_name"),
        )
        .from_as(Alias::new("student_exams"), se.clone())
        .join_as(
            JoinType::LeftJoin,
            Alias::new("students"),
            s.clone(),
            Expr::col((s.clone(), Alias::new("id"))).equals((se.clone(), Alias::new("student_id"))),
        )
        .join_as(
            JoinType::LeftJoin,
            Alias::new("devices"),
            d.clone(),
            Expr::col((d.clone(), Alias::new("ip"))).equals((se.clone(), Alias::new("ip_addr"))),
        )
        .and_where(Expr::col((se.clone(), Alias::new("exam_id"))).eq(exam_id))
        .order_by((s.clone(), Alias::new("created_at")), Order::Desc)
        .to_owned();

    let rows = db.query_all(&query).await?;
    let mut assignments = Vec::with_capacity(rows.len());
    for row in rows {
        assignments.push(student_exam_schema::StudentDeviceAssignDto {
            student_exam_id: row.try_get("", "student_exam_id")?,
            student_id: row.try_get("", "student_id")?,
            student_no: row.try_get("", "student_no")?,
            student_name: row.try_get("", "student_name")?,
            ip_addr: row.try_get("", "ip_addr")?,
            device_name: row.try_get("", "device_name")?,
        });
    }

    Ok(assignments)
}

pub async fn assign_devices_to_student_exams(
    db: &DatabaseConnection,
    exam_id: &str,
    assignments: Vec<student_exam_schema::AssignStudentDeviceItem>,
) -> Result<Vec<student_exam_schema::StudentDeviceAssignDto>> {
    let txn = db.begin().await?;

    for item in assignments {
        StudentExamEntity::update_many()
            .col_expr(Column::IpAddr, Expr::value(item.ip_addr))
            .filter(Column::Id.eq(item.student_exam_id))
            .filter(Column::ExamId.eq(exam_id.to_string()))
            .exec(&txn)
            .await?;
    }

    txn.commit().await?;
    get_student_device_assignments_by_exam_id(db, exam_id).await
}

pub async fn get_student_answer_progress_by_exam_id(
    db: &DatabaseConnection,
    exam_id: &str,
) -> Result<HashMap<String, (i64, i64, i64)>> {
    let sep = Alias::new("sep");
    let query = Query::select()
        .column((sep.clone(), Alias::new("student_id")))
        .column((sep.clone(), Alias::new("answered_count")))
        .column((sep.clone(), Alias::new("total_questions")))
        .column((sep.clone(), Alias::new("progress_percent")))
        .from_as(Alias::new("student_exam_progress"), sep.clone())
        .and_where(Expr::col((sep.clone(), Alias::new("exam_id"))).eq(exam_id))
        .to_owned();

    let rows = db.query_all(&query).await?;
    let mut map = HashMap::with_capacity(rows.len());
    for row in rows {
        let student_id: String = row.try_get("", "student_id")?;
        let answered_count: i64 = row.try_get("", "answered_count")?;
        let total_questions: i64 = row.try_get("", "total_questions")?;
        let progress_percent: i64 = row.try_get("", "progress_percent")?;
        map.insert(student_id, (answered_count, total_questions, progress_percent));
    }

    Ok(map)
}
