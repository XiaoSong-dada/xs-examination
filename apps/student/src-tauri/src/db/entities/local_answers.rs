use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "local_answers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub session_id: String,
    pub question_id: String,
    pub answer: Option<String>,
    pub answer_blob: Option<Vec<u8>>,
    pub revision: i64,
    pub sync_status: String,
    pub last_synced_at: Option<i64>,
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
