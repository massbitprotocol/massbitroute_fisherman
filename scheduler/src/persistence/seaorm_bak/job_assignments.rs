//! SeaORM Entity. Generated by sea-orm-codegen 0.8.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "job_assignments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub job_id: String,
    pub worker_id: String,
    pub plan_id: String,
    pub status: String,
    pub assign_time: i64,
    pub job_name: String,
    pub job_type: String,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef")
    }
}

impl ActiveModelBehavior for ActiveModel {}