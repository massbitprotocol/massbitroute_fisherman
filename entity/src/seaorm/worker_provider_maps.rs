//! SeaORM Entity. Generated by sea-orm-codegen 0.8.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "worker_provider_maps")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub worker_id: String,
    pub provider_id: String,
    pub ping_response_duration: Option<i32>,
    pub ping_timestamp: Option<i64>,
    pub bandwidth: Option<i64>,
    pub bandwidth_timestamp: Option<i64>,
    pub status: Option<i32>,
    pub last_connect_time: Option<i64>,
    pub last_check: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef")
    }
}

impl ActiveModelBehavior for ActiveModel {}
