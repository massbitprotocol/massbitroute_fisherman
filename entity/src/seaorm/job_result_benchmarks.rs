//! SeaORM Entity. Generated by sea-orm-codegen 0.8.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "job_result_benchmarks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub job_id: String,
    pub worker_id: String,
    pub provider_id: String,
    pub provider_type: String,
    pub execution_timestamp: i64,
    pub recorded_timestamp: i64,
    pub request_rate: f64,
    pub transfer_rate: f64,
    pub average_latency: f64,
    pub histogram90: f64,
    pub histogram95: f64,
    pub histogram99: f64,
    pub error_code: i32,
    pub message: String,
    pub response_duration: i32,
    pub plan_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef")
    }
}

impl ActiveModelBehavior for ActiveModel {}
