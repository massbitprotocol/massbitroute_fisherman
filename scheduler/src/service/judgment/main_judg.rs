use crate::models::job_result::ProviderTask;
use crate::persistence::services::job_result_service::JobResultService;

use crate::service::judgment::{get_report_judgments, JudgmentsResult, ReportCheck};
use crate::service::report_portal::{ReportRecord, StoreReport};
use crate::{CONFIG_DIR, IS_REGULAR_REPORT, PORTAL_AUTHORIZATION};
use common::job_manage::JobRole;
use common::jobs::{Job, JobResult};

use common::models::PlanEntity;
use common::{JobId, PlanId, DOMAIN};
use log::{debug, error, info, warn};

use common::util::get_datetime_utc_7;
use serde::{Deserialize, Serialize};

use crate::models::workers::WorkerInfoStorage;
use crate::service::delivery::CancelPlanBuffer;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;

#[derive(Default)]
pub struct MainJudgment {
    _result_service: Arc<JobResultService>,
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
        values.get(key).cloned()
    }
}

impl MainJudgment {
    pub fn new(result_service: Arc<JobResultService>, phase: &JobRole) -> Self {
        let judgments = get_report_judgments(CONFIG_DIR.as_str(), result_service.clone(), phase);
        MainJudgment {
            _result_service: result_service,
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
        self.judgment_result_cache.get_value(&key)
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
                currentjob_result = match judgment.apply_for_results(provider_task, &results).await
                {
                    Ok(result) => result,
                    Err(err) => JudgmentsResult::new_failed(
                        provider_task.task_name.clone(),
                        format!(
                            "Verify judgement {} apply for {} got error: {:?}",
                            judgment.get_name(),
                            provider_task.task_name,
                            err
                        ),
                    ),
                };

                info!(
                    "Verify judgment {} result {:?} for provider {:?} with results {results:?}",
                    judgment.get_name(),
                    &currentjob_result,
                    provider_task,
                );
                total_result.insert(job_id.clone(), currentjob_result.clone());
                break;
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
        worker_pool: Arc<WorkerInfoStorage>,
        cancel_plans_buffer: Arc<TokioMutex<CancelPlanBuffer>>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        if results.is_empty() {
            return Ok(JudgmentsResult::Unfinished);
        }
        // This unwrap is safe because results is not empty.
        // All worker_id and plan in results should be the same.
        let worker_id = results.get(0).unwrap().worker_id.clone();
        let plan_id = results.get(0).unwrap().plan_id.clone();

        let mut judg_result = JudgmentsResult::Unfinished;
        //Separate jobs by job name
        let provider_type = results
            .get(0)
            .map(|res| res.provider_type.clone())
            .unwrap_or_default();

        for judgment in self.judgments.iter() {
            if !judgment.can_apply_for_result(provider_task) {
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
                    judg_result = JudgmentsResult::new_failed(
                        provider_task.task_name.clone(),
                        format!(
                            "Regular judgement {} apply for {} got error: {:?}",
                            judgment.get_name(),
                            provider_task.task_name,
                            err
                        ),
                    );
                }
            }

            debug!(
                "Regular judgment {} result {:?} on task {} for provider {:?}",
                judgment.get_name(),
                &judg_result,
                provider_task.task_name,
                provider_task.provider_id
            );
            if let JudgmentsResult::Failed(_) = &judg_result {
                let mut report = StoreReport::build(
                    &"Scheduler".to_string(),
                    &JobRole::Regular,
                    &*PORTAL_AUTHORIZATION,
                    &DOMAIN,
                );
                report.set_report_data_short(
                    &judg_result,
                    &provider_task.provider_id,
                    &provider_type,
                );
                if *IS_REGULAR_REPORT {
                    debug!("*** Send regular report to portal:{:?}", report);
                    let res = report.send_data().await;
                    info!("Send report to portal res: {:?}", res);
                    match res {
                        Ok(resp) => {
                            if resp.status().is_success() {
                                // Remove job plan in worker because the provider is failed
                                if let Some(worker) = worker_pool.get_worker(worker_id).await {
                                    // let res = worker.send_cancel_plans(&vec![plan_id]).await;
                                    // if let Err(err) = res {
                                    //     error!("send_cancel_plans error: {:?}", err);
                                    // }
                                    cancel_plans_buffer
                                        .lock()
                                        .await
                                        .insert_plan(plan_id, worker);
                                }
                            } else {
                                error!(
                                    "Portal response error code: {:?} and body: {:?}",
                                    resp.status(),
                                    resp.text().await
                                );
                            }
                        }
                        Err(err) => {
                            error!("Error: {:?}", &err);
                        }
                    };
                } else {
                    //let result = json!({"report_time":get_datetime_utc_7(),"provider_task":provider_task,"result":judg_result});
                    let report_record = ReportRecord::new(
                        get_datetime_utc_7(),
                        provider_task.provider_id.clone(),
                        "".to_string(),
                        judg_result.clone(),
                    );
                    let res = report.write_data(report_record);
                    info!("*** Write regular report to file res: {:?}", res);
                }
            }
            break;
        }

        Ok(judg_result)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use anyhow::Error;
    use common::component::ComponentType;
    use common::util::get_current_time;
    use common::BlockChainType;
    use log::info;

    use test_util::helper::{load_env, mock_db_connection, mock_job, mock_job_result, JobName};

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
            BlockChainType::Eth,
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
            BlockChainType::Dot,
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
        let cancel_plans_buffer: Arc<TokioMutex<CancelPlanBuffer>> =
            Arc::new(TokioMutex::new(CancelPlanBuffer::default()));
        // Test apply_for_results
        let _worker_infos = Arc::new(WorkerInfoStorage::default());
        assert_eq!(
            judge
                .apply_for_regular(
                    &task_benchmark,
                    &vec![],
                    _worker_infos.clone(),
                    cancel_plans_buffer.clone()
                )
                .await?,
            JudgmentsResult::Unfinished
        );

        // For eth
        let job_result_eth = mock_job_result(
            &JobName::RoundTripTime,
            BlockChainType::Eth,
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
                _worker_infos,
                cancel_plans_buffer.clone(),
            )
            .await?;
        println!("Judge Eth res: {:?}", res);
        assert_eq!(res, JudgmentsResult::Pass,);

        // For dot
        let job_result_dot = mock_job_result(
            &JobName::LatestBlock,
            BlockChainType::Dot,
            "job_latest_block",
            phase.clone(),
        );
        info!("job_result: {:?}", job_result_dot);
        let _worker_infos = Arc::new(WorkerInfoStorage::default());
        let res = judge
            .apply_for_regular(
                &task_latest_block,
                &vec![job_result_dot],
                _worker_infos.clone(),
                cancel_plans_buffer.clone(),
            )
            .await?;
        println!("Judge Dot res: {:?}", res);
        assert_eq!(res, JudgmentsResult::Pass,);

        // For dot
        let job_result_dot = mock_job_result(
            &JobName::LatestBlock,
            BlockChainType::Eth,
            "job_latest_block",
            phase.clone(),
        );
        info!("job_result: {:?}", job_result_dot);
        let res = judge
            .apply_for_regular(
                &task_latest_block,
                &vec![job_result_dot],
                _worker_infos,
                cancel_plans_buffer.clone(),
            )
            .await?;
        println!("Judge Dot res: {:?}", res);
        assert_eq!(res, JudgmentsResult::Pass,);

        Ok(())
    }
}
