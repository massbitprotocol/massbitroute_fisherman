//! SeaORM Entity. Generated by sea-orm-codegen 0.8.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "job_result_latest_blocks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub job_id: String,
    pub worker_id: String,
    pub provider_id: String,
    pub provider_type: String,
    pub execution_timestamp: i64,
    pub chain_id: String,
    pub block_number: i64,
    pub block_timestamp: i64,
    pub plan_id: String,
    pub block_hash: String,
    pub http_code: i32,
    pub error_code: i32,
    pub message: String,
    pub response_duration: i64,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef")
    }
}

impl ActiveModelBehavior for ActiveModel {}
