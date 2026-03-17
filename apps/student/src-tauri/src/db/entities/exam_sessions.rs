use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "exam_sessions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub exam_id: String,
    pub student_id: String,
    pub student_no: String,
    pub student_name: String,
    pub assigned_ip_addr: String,
    pub assigned_device_name: Option<String>,
    pub exam_title: String,
    pub status: String,
    pub assignment_status: String,
    pub started_at: Option<i64>,
    pub ends_at: Option<i64>,
    pub paper_version: Option<String>,
    pub encryption_nonce: Option<Vec<u8>>,
    pub last_synced_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match *self {}
    }
}

impl ActiveModelBehavior for ActiveModel {}
