use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "teacher_endpoints")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub endpoint: String,
    pub name: Option<String>,
    pub remark: Option<String>,
    pub is_master: i32,
    pub last_seen: Option<i64>,
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
