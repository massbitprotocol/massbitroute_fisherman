//! SeaORM Entity. Generated by sea-orm-codegen 0.8.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "provider_latest_blocks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub provider_id: String,
    pub blockchain: String,
    pub network: String,
    pub block_number: Option<i64>,
    pub block_timestamp: Option<i64>,
    pub block_hash: Option<String>,
    pub response_timestamp: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef")
    }
}

impl ActiveModelBehavior for ActiveModel {}
