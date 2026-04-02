use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "exam_snapshots")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub session_id: String,
    pub exam_meta: Vec<u8>,
    pub questions_payload: Vec<u8>,
    pub downloaded_at: i64,
    pub expires_at: Option<i64>,
    pub package_path: Option<String>,
    pub package_status: Option<String>,
    pub package_batch_id: Option<String>,
    pub package_sha256: Option<String>,
    pub package_received_at: Option<i64>,
    pub assets_sync_status: Option<String>,
    pub assets_synced_at: Option<i64>,
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
