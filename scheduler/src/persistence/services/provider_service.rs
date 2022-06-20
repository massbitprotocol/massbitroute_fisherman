use crate::persistence::seaorm::worker_provider_maps;
use crate::persistence::ProviderMapModel;
use anyhow::anyhow;
use common::component::Zone;
use common::workers::WorkerInfo;
use log::error;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Value};
use sea_orm::{DatabaseBackend, DatabaseConnection, ExecResult, Statement};
use std::fmt::Error;
use std::str::FromStr;
use std::sync::Arc;

const TABLE_NAME: &str = "worker_provider_connections";
const INSERT_RESPONSE_TIME_QUERY: &str = r#"INSERT INTO worker_provider_connections
                                (worker_id, provider_id, ping_response_time, ping_time, last_connect_time, last_check)"#;
const CONFLICT_RESPONSE_TIME_QUERY: &str = r#"ON CONFLICT (worker_provider_connections_provider_worker_uindex)
                                DO UPDATE SET ping_response_time = EXCLUDED.ping_response_time
                                              ,ping_time= EXCLUDED.ping_time
                                              ,last_connect_time = EXCLUDED.last_connect_time
                                              ,last_check = EXCLUDED.last_check;"#;
#[derive(Default)]
pub struct ProviderService {
    db: Arc<DatabaseConnection>,
}
impl ProviderService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        ProviderService { db }
    }
}

impl ProviderService {
    pub async fn get_provider_maps(&self) -> Vec<ProviderMapModel> {
        worker_provider_maps::Entity::find()
            .all(self.db.as_ref())
            .await
            .unwrap_or(vec![])
    }
    pub async fn store_provider_maps(
        &self,
        provider_maps: &Vec<ProviderMapModel>,
    ) -> Result<u64, anyhow::Error> {
        let mut values = Vec::new();
        for item in provider_maps {
            values.push(Value::from(item.worker_id.clone()));
            values.push(Value::from(item.provider_id.clone()));
            values.push(Value::from(item.ping_response_time.clone()));
            values.push(Value::from(item.ping_time.clone()));
            values.push(Value::from(item.last_connect_time.clone()));
            values.push(Value::from(item.last_check.clone()))
        }
        let mut place_holders = Vec::new();
        for i in 1..=values.len() {
            place_holders.push(format!("${}", i));
        }
        let query = format!(
            "{} VALUES ({}) {}",
            INSERT_RESPONSE_TIME_QUERY,
            place_holders.join(","),
            CONFLICT_RESPONSE_TIME_QUERY
        );
        println!("{}", query.as_str());
        let exec_res: ExecResult = self
            .db
            .as_ref()
            .execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                query.as_str(),
                values,
            ))
            .await?;
        Ok(exec_res.rows_affected())
    }
}
