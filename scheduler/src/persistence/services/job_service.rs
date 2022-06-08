use crate::persistence::seaorm::jobs;
use crate::persistence::seaorm::jobs::Model;
use anyhow::anyhow;
use common::component::Zone;
use common::job_manage::Job;
use common::worker::WorkerInfo;
use log::{debug, error, log};
use sea_orm::DatabaseConnection;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Default)]
pub struct JobService {
    db: Arc<DatabaseConnection>,
}
impl JobService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        JobService { db }
    }
    pub async fn save_jobs(&self, vec_jobs: &Vec<Job>) -> Result<i32, anyhow::Error> {
        let records = vec_jobs
            .iter()
            .map(|job| jobs::ActiveModel::from(job))
            .collect::<Vec<jobs::ActiveModel>>();
        let length = records.len();
        debug!("save_jobs records:{:?}", records);

        match jobs::Entity::insert_many(records)
            .exec(self.db.as_ref())
            .await
        {
            Ok(res) => {
                log::debug!("Insert many records {:?}", length);
                Ok(res.last_insert_id)
            }
            Err(err) => {
                log::debug!("Error {:?}", &err);
                Err(anyhow!("{:?}", &err))
            }
        }
    }
}
/*
let apple = fruit::ActiveModel {
name: Set("Apple".to_owned()),
..Default::default()
};

let orange = fruit::ActiveModel {
name: Set("Orange".to_owned()),
..Default::default()
};

let res: InsertResult = Fruit::insert_many(vec![apple, orange]).exec(db).await?;
assert_eq!(res.last_insert_id, 30)
 */
