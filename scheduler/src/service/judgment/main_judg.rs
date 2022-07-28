use crate::models::job_result::ProviderTask;
use crate::persistence::services::job_result_service::JobResultService;

use crate::service::judgment::{get_report_judgments, JudgmentsResult, ReportCheck};
use crate::service::report_portal::StoreReport;
use crate::{CONFIG, CONFIG_DIR, IS_REGULAR_REPORT, IS_VERIFY_REPORT, PORTAL_AUTHORIZATION};
use common::job_manage::JobRole;
use common::jobs::{Job, JobResult};

use common::models::PlanEntity;
use common::{JobId, PlanId, DOMAIN};
use log::{debug, info, warn};

use anyhow::Error;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct MainJudgment {
    result_service: Arc<JobResultService>,
    judgments: Vec<Arc<dyn ReportCheck>>,
    judgment_result_cache: LatestJudgmentCache,
}
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash, Default)]
pub struct JudgmentKey {
    pub plan_id: PlanId,
    pub job_id: JobId,
}

impl JudgmentKey {
    pub fn new(plan_id: PlanId, job_id: JobId) -> JudgmentKey {
        Self { plan_id, job_id }
    }
}
#[derive(Debug, Default)]
pub struct LatestJudgmentCache {
    values: Mutex<HashMap<JudgmentKey, JudgmentsResult>>,
}

impl LatestJudgmentCache {
    pub fn insert_value(&self, plan_id: PlanId, job_id: JobId, judg_result: JudgmentsResult) {
        let key = JudgmentKey::new(plan_id, job_id);
        let mut map = self.values.lock().unwrap();
        map.insert(key, judg_result);
    }
    pub fn get_value(&self, key: &JudgmentKey) -> Option<JudgmentsResult> {
        let values = self.values.lock().unwrap();
        values.get(key).map(|r| r.clone())
    }
}

impl MainJudgment {
    pub fn new(result_service: Arc<JobResultService>, phase: &JobRole) -> Self {
        let judgments = get_report_judgments(CONFIG_DIR.as_str(), result_service.clone(), phase);
        MainJudgment {
            result_service,
            judgments,
            judgment_result_cache: Default::default(),
        }
    }
    pub fn put_judgment_result(&self, plan: &PlanEntity, job_id: JobId, result: JudgmentsResult) {
        self.judgment_result_cache
            .insert_value(plan.plan_id.clone(), job_id.clone(), result);
    }
    pub fn get_latest_judgment(
        &self,
        plan: &PlanEntity,
        job_id: &JobId,
    ) -> Option<JudgmentsResult> {
        let key = JudgmentKey::new(plan.plan_id.clone(), job_id.clone());
        self.judgment_result_cache
            .get_value(&key)
            .map(|r| r.clone())
    }
    /*
     * for each plan, check result of all judgment,
     * For judgment wich can apply for input provider task, do judgment base on input result,
     * For other judgments get result by plan
     */
    pub async fn apply_for_verify(
        &self,
        provider_task: &ProviderTask,
        plan: &PlanEntity,
        results: &Vec<JobResult>,
        plan_jobs: &Vec<Job>,
    ) -> Result<HashMap<JobId, JudgmentsResult>, anyhow::Error> {
        if plan_jobs.is_empty() {
            return Ok(HashMap::new());
        }
        //Judge received results
        debug!(
            "Make judgment for {} job results of plan {} with provider {:?} and {} jobs",
            results.len(),
            &plan.plan_id,
            provider_task,
            plan_jobs.len()
        );
        let mut total_result = HashMap::<JobId, JudgmentsResult>::new();
        let mut currentjob_result = JudgmentsResult::Unfinished;
        let job_id = results
            .get(0)
            .map(|job_result| job_result.job_id.clone())
            .unwrap_or_default();
        for judgment in self.judgments.iter() {
            if judgment.can_apply_for_result(provider_task) {
                currentjob_result = judgment
                    .apply_for_results(provider_task, &results)
                    .await
                    .unwrap_or(JudgmentsResult::Failed);
                info!(
                    "Verify judgment {} result {:?} for provider {:?} with results {results:?}",
                    judgment.get_name(),
                    &currentjob_result,
                    provider_task,
                );
                total_result.insert(job_id.clone(), currentjob_result.clone());
            };
        }

        //Put judgment result to cache
        self.put_judgment_result(plan, job_id, currentjob_result.clone());
        //Input plan_result as result of current job
        for job in plan_jobs {
            match self.get_latest_judgment(plan, &job.job_id) {
                Some(latest_result) => {
                    debug!(
                        "Latest result of plan {:?}, job {} {:?} is {:?}",
                        &plan.plan_id, &job.job_name, &job.job_id, &latest_result
                    );
                    total_result.insert(job.job_id.clone(), latest_result);
                }
                None => {
                    debug!(
                        "Result not found for plan {:?} and job {:?}",
                        &plan.plan_id, &job.job_id
                    );
                    total_result.insert(job.job_id.clone(), JudgmentsResult::Unfinished);
                }
            }
        }
        //If all task pass then plan result is Pass
        Ok(total_result)
    }

    /*
     * Use for regular judgment
     */
    pub async fn apply_for_regular(
        &self,
        provider_task: &ProviderTask,
        results: &Vec<JobResult>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        if results.is_empty() {
            return Ok(JudgmentsResult::Unfinished);
        }
        let mut judg_result = JudgmentsResult::Unfinished;
        //Separate jobs by job name
        let provider_type = results
            .get(0)
            .map(|res| res.provider_type.clone())
            .unwrap_or_default();

        for judgment in self.judgments.iter() {
            if !judgment.can_apply_for_result(&provider_task) {
                continue;
            }
            let res = judgment.apply_for_results(provider_task, results).await;
            match res {
                Ok(res) => {
                    judg_result = res;
                }
                Err(err) => {
                    warn!(
                        "Judge {} for results {:?} return error: {}, ",
                        judgment.get_name(),
                        results,
                        err
                    );
                    judg_result = JudgmentsResult::Error;
                }
            }

            debug!(
                "Regular judgment {} result {:?} on task {} for provider {:?}",
                judgment.get_name(),
                &judg_result,
                provider_task.task_name,
                provider_task.provider_id
            );
            match judg_result {
                JudgmentsResult::Failed | JudgmentsResult::Error => {
                    let mut report = StoreReport::build(
                        &"Scheduler".to_string(),
                        &JobRole::Regular,
                        &*PORTAL_AUTHORIZATION,
                        &DOMAIN,
                    );
                    report.set_report_data_short(false, &provider_task.provider_id, &provider_type);
                    if *IS_REGULAR_REPORT {
                        debug!("Send plan report to portal:{:?}", report);
                        let res = report.send_data().await;
                        info!("Send report to portal res: {:?}", res);
                    } else {
                        let result = json!({"provider_task":provider_task,"result":judg_result});
                        let res = report.write_data(result);
                        info!("Write report to file res: {:?}", res);
                    }
                }
                _ => {}
            }
            break;
        }

        Ok(judg_result)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::CONFIG_DIR;
    use anyhow::Error;
    use common::component::ComponentType;
    use common::util::get_current_time;
    use log::info;
    use sea_orm::{DatabaseBackend, MockDatabase};

    use test_util::helper::JobName::Benchmark;
    use test_util::helper::{
        init_logging, load_env, mock_db_connection, mock_job, mock_job_result, ChainTypeForTest,
        JobName,
    };

    #[tokio::test]
    async fn test_main_judgment_verify_and_regular() -> Result<(), Error> {
        load_env();
        //init_logging();
        let db_conn = mock_db_connection();
        let result_service = JobResultService::new(Arc::new(db_conn));
        let phase = JobRole::Verification;
        let judge = MainJudgment::new(Arc::new(result_service.clone()), &phase);

        let task_benchmark = ProviderTask::new(
            "provider_id".to_string(),
            ComponentType::Node,
            "Benchmark".to_string(),
            "VerifyEthNode".to_string(),
        );
        let task_latest_block = ProviderTask::new(
            "provider_id".to_string(),
            ComponentType::Node,
            "HttpRequest".to_string(),
            "LatestBlock".to_string(),
        );
        let task_rtt = ProviderTask::new(
            "provider_id".to_string(),
            ComponentType::Node,
            "HttpRequest".to_string(),
            "RoundTripTime".to_string(),
        );
        let plan = PlanEntity::new(
            "provider_id".to_string(),
            get_current_time(),
            1000,
            phase.to_string(),
        );

        let job_rtt = mock_job(&JobName::RoundTripTime, "url", "job_RoundTripTime", &phase);
        let job_benchmark = mock_job(&JobName::Benchmark, "url", "job_Benchmark", &phase);
        let job_latest_block = mock_job(&JobName::LatestBlock, "url", "job_LatestBlock", &phase);

        //////////////// For Verification /////////////////
        // Test apply_for_results
        assert_eq!(
            judge
                .apply_for_verify(&task_benchmark, &plan, &vec![], &vec![])
                .await?,
            HashMap::default()
        );

        // For eth
        let job_result_eth = mock_job_result(
            &JobName::Benchmark,
            ChainTypeForTest::Eth,
            "job_Benchmark",
            phase.clone(),
        );
        info!("job_result: {:?}", job_result_eth);
        let res = judge
            .apply_for_verify(
                &task_benchmark,
                &plan,
                &vec![job_result_eth.clone()],
                &vec![
                    job_rtt.clone(),
                    job_benchmark.clone(),
                    job_latest_block.clone(),
                ],
            )
            .await?;
        println!("Judge Eth res: {:?}", res);
        assert_eq!(
            res,
            HashMap::from([
                ("job_RoundTripTime".to_string(), JudgmentsResult::Unfinished),
                ("job_Benchmark".to_string(), JudgmentsResult::Pass),
                ("job_LatestBlock".to_string(), JudgmentsResult::Unfinished),
            ])
        );

        // For dot
        let job_result_dot = mock_job_result(
            &JobName::Benchmark,
            ChainTypeForTest::Dot,
            "job_Benchmark",
            phase.clone(),
        );
        info!("job_result: {:?}", job_result_dot);
        let res = judge
            .apply_for_verify(
                &task_benchmark,
                &plan,
                &vec![job_result_dot],
                &vec![
                    job_rtt.clone(),
                    job_benchmark.clone(),
                    job_latest_block.clone(),
                ],
            )
            .await?;
        println!("Judge Dot res: {:?}", res);
        assert_eq!(
            res,
            HashMap::from([
                ("job_RoundTripTime".to_string(), JudgmentsResult::Unfinished),
                ("job_Benchmark".to_string(), JudgmentsResult::Pass),
                ("job_LatestBlock".to_string(), JudgmentsResult::Unfinished),
            ])
        );

        //////////////// For Regular /////////////////
        let phase = JobRole::Regular;
        let judge = MainJudgment::new(Arc::new(result_service), &phase);

        // Test apply_for_results
        assert_eq!(
            judge.apply_for_regular(&task_benchmark, &vec![]).await?,
            JudgmentsResult::Unfinished
        );

        // For eth
        let job_result_eth = mock_job_result(
            &JobName::RoundTripTime,
            ChainTypeForTest::Eth,
            "job_rtt",
            phase.clone(),
        );
        info!("job_result: {:?}", job_result_eth);
        let res = judge
            .apply_for_regular(
                &task_rtt,
                &vec![
                    job_result_eth.clone(),
                    job_result_eth.clone(),
                    job_result_eth.clone(),
                    job_result_eth.clone(),
                    job_result_eth.clone(),
                    job_result_eth.clone(),
                    job_result_eth.clone(),
                ],
            )
            .await?;
        println!("Judge Eth res: {:?}", res);
        assert_eq!(res, JudgmentsResult::Pass,);

        // For dot
        let job_result_dot = mock_job_result(
            &JobName::LatestBlock,
            ChainTypeForTest::Dot,
            "job_latest_block",
            phase.clone(),
        );
        info!("job_result: {:?}", job_result_dot);
        let res = judge
            .apply_for_regular(&task_latest_block, &vec![job_result_dot])
            .await?;
        println!("Judge Dot res: {:?}", res);
        assert_eq!(res, JudgmentsResult::Pass,);

        // For dot
        let job_result_dot = mock_job_result(
            &JobName::LatestBlock,
            ChainTypeForTest::Eth,
            "job_latest_block",
            phase.clone(),
        );
        info!("job_result: {:?}", job_result_dot);
        let res = judge
            .apply_for_regular(&task_latest_block, &vec![job_result_dot])
            .await?;
        println!("Judge Dot res: {:?}", res);
        assert_eq!(res, JudgmentsResult::Pass,);

        Ok(())
    }
}
