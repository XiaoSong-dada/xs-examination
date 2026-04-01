use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "exam_question_assets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub session_id: String,
    pub exam_id: String,
    pub question_id: String,
    pub scope: String,
    pub asset_local_path: String,
    pub source_archive_path: Option<String>,
    pub checksum: Option<String>,
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
