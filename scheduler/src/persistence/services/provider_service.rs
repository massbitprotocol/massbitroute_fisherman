use crate::persistence::ProviderMapModel;

use entity::worker_provider_maps;
use log::debug;
use sea_orm::{ConnectionTrait, EntityTrait, Value};
use sea_orm::{DatabaseBackend, DatabaseConnection, Statement};
use std::collections::HashMap;

use std::sync::Arc;

//const TABLE_NAME: &str = "worker_provider_maps";
const INSERT_RESPONSE_TIME_QUERY: &str = r#"INSERT INTO worker_provider_maps
                                (worker_id, provider_id, ping_response_duration, ping_timestamp, last_connect_time, last_check)"#;
const CONFLICT_RESPONSE_TIME_QUERY: &str = r#"ON CONFLICT ON CONSTRAINT worker_provider_maps_worker_provider_key
                                DO UPDATE SET ping_response_duration = EXCLUDED.ping_response_duration
                                              ,ping_timestamp= EXCLUDED.ping_timestamp
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
            .unwrap_or_default()
    }
    pub async fn store_provider_maps(
        &self,
        provider_maps: &Vec<ProviderMapModel>,
    ) -> Result<u64, anyhow::Error> {
        let mut values = Vec::new();
        let mut place_holders = Vec::new();
        let mut map = HashMap::new(); //Map worker <-> provider
        let mut row = 1;
        let column_count = 6;
        for item in provider_maps {
            if item.provider_id.as_str()
                == map
                    .get(&item.worker_id)
                    .unwrap_or(&String::default())
                    .as_str()
            {
                continue;
            }
            map.insert(item.worker_id.clone(), item.provider_id.clone());
            let mut rows = Vec::new();
            values.push(Value::from(item.worker_id.clone()));
            values.push(Value::from(item.provider_id.clone()));
            values.push(Value::from(item.ping_response_duration));
            values.push(Value::from(item.ping_timestamp));
            values.push(Value::from(item.last_connect_time));
            values.push(Value::from(item.last_check));
            for i in 0..column_count {
                rows.push(format!("${}", row + i));
            }
            row += column_count;
            place_holders.push(format!("({})", rows.join(",")));
        }
        let query = format!(
            "{} VALUES {} {}",
            INSERT_RESPONSE_TIME_QUERY,
            place_holders.join(","),
            CONFLICT_RESPONSE_TIME_QUERY
        );
        debug!("{}", query.as_str());
        match self
            .db
            .as_ref()
            .execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                query.as_str(),
                values,
            ))
            .await
        {
            Ok(exec_res) => {
                log::debug!("{:?}", &exec_res);
                Ok(exec_res.rows_affected())
            }
            Err(err) => {
                log::error!("Upsert error: {:?}", &err);
                Ok(0)
            }
        }
    }
}
