use futures::pin_mut;
use futures_util::future::join_all;
use minifier::json::minify;

use serde::{Deserialize, Serialize};

use handlebars::Handlebars;
use log::{debug, info, warn};
use serde_json::Value;

use std::collections::HashMap;

use std::fs::File;
use std::io::{BufRead, BufReader};

use anyhow::Error;
use reqwest::RequestBuilder;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::{thread, usize};

use crate::job::check_module::CheckMkStatus::{Unknown, Warning};
use crate::{BASE_ENDPOINT_JSON, BENCHMARK_WRK_PATH, LOCAL_IP, PORTAL_AUTHORIZATION, ZONE};
use common::component::ComponentType;
use common::component::{ComponentInfo, Zone};
use common::job_action::{CheckMkStatus, CheckStep, EndpointInfo};
use common::job_manage::Config;
use std::str::FromStr;
use strum_macros::EnumString;
use warp::{Rejection, Reply};
pub use wrap_wrk::{WrkBenchmark, WrkReport};

type BlockChainType = String;
type NetworkType = String;
type UrlType = String;
type TaskType = String;
type StepResult = HashMap<String, String>;
type ComponentId = String;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ActionCall {
    action_type: String,
    is_base_node: bool,
    request_type: String,
    header: HashMap<String, String>,
    body: String,
    time_out: usize,
    return_fields: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ActionCompare {
    action_type: String,
    operator_items: OperatorCompare,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct OperatorCompare {
    operator_type: String,
    params: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct UserInfo {
    pub name: String,
    pub id: String,
    pub email: String,
    pub verified: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct CheckComponent {
    // MBR Domain
    pub domain: String,
    // inner data
    pub base_nodes: HashMap<BlockChainType, HashMap<NetworkType, Vec<EndpointInfo>>>,
    pub config: Config,
}

type CheckFlows = HashMap<TaskType, Vec<CheckFlow>>;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct CheckFlow {
    #[serde(default)]
    blockchain: BlockChainType,
    #[serde(default)]
    component: String,
    #[serde(default)]
    check_steps: Vec<CheckStep>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct FailedCase {
    #[serde(default)]
    critical: bool,
    #[serde(default)]
    message: String,
    #[serde(default)]
    conclude: CheckMkStatus,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct CheckMkReport {
    pub status: u8,
    pub service_name: String,
    pub metric: CheckMkMetric,
    pub status_detail: String,
    pub success: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct CheckMkMetric {
    pub metric: HashMap<String, Value>,
}

impl ToString for CheckMkMetric {
    fn to_string(&self) -> String {
        if self.metric.is_empty() {
            format!("-")
        } else {
            self.metric
                .iter()
                .map(|(key, val)| format!("{}={}", key, val))
                .collect::<Vec<String>>()
                .join("|")
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ActionResponse {
    pub success: bool,
    pub conclude: CheckMkStatus,
    pub return_name: String,
    pub result: HashMap<String, String>,
    pub message: String,
}

impl ToString for CheckMkReport {
    fn to_string(&self) -> String {
        format!(
            r#"{status} {service_name} {metric} {status_detail}"#,
            status = &self.status,
            service_name = &self.service_name,
            metric = self.metric.to_string(),
            status_detail = self.status_detail
        )
    }
}

impl CheckMkReport {
    fn from_wrk_report(
        wrk_report: WrkReport,
        success_percent_threshold: u32,
        response_time_threshold_ms: f32,
        accepted_percent_low_latency: f32,
    ) -> Self {
        let mut status = CheckMkStatus::Ok as u8;
        let mut message = String::new();
        let latency = wrk_report
            .latency
            .avg
            .and_then(|avg| Some(avg.as_millis()))
            .unwrap_or(u128::MAX);
        let success_percent = wrk_report.get_success_percent().unwrap();
        message.push_str(
            format!(
                "Percent of requests that latency lower than {}ms: {}%, average latency: {}ms, request success percent: {}%.",
                response_time_threshold_ms, wrk_report.percent_low_latency*100f32, latency, success_percent
            )
            .as_str(),
        );

        if accepted_percent_low_latency > wrk_report.percent_low_latency {
            status = CheckMkStatus::Critical as u8;
            message.push_str(
                format!(
                    "False because percent low latency request is too low: {}% (required percent {}%). ",
                    wrk_report.percent_low_latency*100f32, accepted_percent_low_latency*100f32
                )
                .as_str(),
            );
        }

        if success_percent < success_percent_threshold {
            status = CheckMkStatus::Critical as u8;
            message.push_str(
                format!(
                    "False because of request success percent is too low: {}% (required percent {}%). ",
                    success_percent, success_percent_threshold
                )
                .as_str(),
            );
        }

        CheckMkReport {
            status,
            service_name: "benchmark".to_string(),
            metric: Default::default(),
            status_detail: message,
            success: true,
        }
    }
}

impl CheckMkReport {
    fn new_failed_report(msg: String) -> Self {
        let mut resp = CheckMkReport::default();
        resp.success = false;
        resp.status = CheckMkStatus::Critical as u8;
        resp.status_detail = msg.to_string();
        resp
    }
    fn new_unknown_report(msg: String) -> Self {
        let mut resp = CheckMkReport::default();
        resp.success = true;
        resp.status = CheckMkStatus::Unknown as u8;
        resp.status_detail = msg.to_string();
        resp
    }
    pub fn is_component_status_ok(&self) -> bool {
        //Fixme: currently count unknown status as success call. It should separate into 2 cases.
        ((self.success == true) && (self.status == 0)) || (self.status == 4)
    }
    pub fn combine_report(logic_check: &CheckMkReport, benchmark_check: &CheckMkReport) -> Self {
        let mut status = CheckMkStatus::Ok;
        if logic_check.status == CheckMkStatus::Critical as u8
            || benchmark_check.status == CheckMkStatus::Critical as u8
        {
            status = CheckMkStatus::Critical
        } else if logic_check.status == CheckMkStatus::Unknown as u8
            || logic_check.status == CheckMkStatus::Unknown as u8
        {
            status = CheckMkStatus::Unknown
        } else if logic_check.status == CheckMkStatus::Warning as u8
            || logic_check.status == CheckMkStatus::Warning as u8
        {
            status = CheckMkStatus::Warning
        }

        // assert_eq!(logic_check.service_name, benchmark_check.service_name);
        let service_name = logic_check.service_name.clone();

        let mut metric = logic_check.metric.clone();
        metric.metric.extend(benchmark_check.metric.metric.clone());

        let mut status_detail = format!(
            "Logic check:{} Benchmark check:{}",
            logic_check.status_detail, benchmark_check.status_detail
        );
        let success = logic_check.success && benchmark_check.success;
        CheckMkReport {
            status: status as u8,
            service_name,
            metric,
            status_detail,
            success,
        }
    }
}

impl CheckComponent {
    pub fn builder() -> GeneratorBuilder {
        GeneratorBuilder::default()
    }

    fn do_compare(
        operator: &OperatorCompare,
        step_result: &StepResult,
    ) -> Result<bool, anyhow::Error> {
        match operator.operator_type.as_str() {
            "and" => {
                let sub_actions: Vec<OperatorCompare> =
                    serde_json::from_value(operator.params.clone())?;
                let mut result = true;
                for sub_operator in sub_actions {
                    result = result && CheckComponent::do_compare(&sub_operator, step_result)?;
                }
                return Ok(result);
            }
            "eq" => {
                let items: Vec<String> = serde_json::from_value(operator.params.clone())?;
                let mut item_values = Vec::new();
                for mut item in items {
                    if item.starts_with("#") {
                        item.remove(0);
                        item_values.push(item.clone());
                    } else {
                        let item_value = step_result
                            .get(item.as_str())
                            .ok_or(anyhow::Error::msg("Cannot find key"))?;
                        item_values.push(item_value.clone());
                    }
                }
                debug!("item_values: {:?}", item_values);
                if item_values.len() > 1 {
                    let first = &item_values[0];
                    for item_value in item_values.iter() {
                        if item_value != first {
                            return Ok(false);
                        }
                    }
                    return Ok(true);
                } else {
                    return Ok(false);
                }
            }
            _ => return Err(anyhow::Error::msg("Cannot find key")),
        }
    }

    fn compare_action(
        &self,
        action: &ActionCompare,
        _node: &ComponentInfo,
        return_name: &String,
        step_result: &StepResult,
    ) -> Result<ActionResponse, anyhow::Error> {
        debug!(
            "Run Compare Action: {:?}, step_result: {:?}",
            action, step_result
        );

        let success = CheckComponent::do_compare(&action.operator_items, step_result)?;

        debug!("Run Compare Action success: {}", success);

        let mut result: HashMap<String, String> = HashMap::new();
        result.insert(return_name.clone(), success.to_string());
        return Ok(ActionResponse {
            success: true,
            conclude: match success {
                true => CheckMkStatus::Ok,
                false => CheckMkStatus::Unknown,
            },
            return_name: return_name.clone(),
            result,
            message: format!("compare items: {:?}", action.operator_items),
        });
    }

    fn replace_string(org: String, step_result: &StepResult) -> Result<String, anyhow::Error> {
        let handlebars = Handlebars::new();
        handlebars
            .render_template(org.as_str(), step_result)
            .map_err(|err| {
                println!("err:{:?}", err);
                anyhow::Error::msg(format!("{}", err))
            })
    }

    pub async fn call_action(
        &self,
        component: &ComponentInfo,
        step_result: &StepResult,
        step: &CheckStep,
    ) -> Result<ActionResponse, anyhow::Error> {
        let action: ActionCall = serde_json::from_value(step.action.clone()).unwrap();
        debug!("call action: {:?}", &action);

        if action.is_base_node {
            // Get base_endpoints
            let mut report = Err(anyhow::Error::msg("Cannot found working base node"));
            let base_endpoints = self
                .base_nodes
                .get(&component.blockchain)
                .ok_or(anyhow::Error::msg(format!(
                    "Cannot found base node for chain {:?}",
                    &component.blockchain
                )))?
                .get(&component.network)
                .ok_or(anyhow::Error::msg(format!(
                    "Cannot found base node for network {:?}",
                    &component.network
                )))?;
            debug!("base endpoints: {:?}", &base_endpoints);
            // Calling to Base node retry if failed
            for endpoint in base_endpoints {
                debug!("try endpoint:{:?}", endpoint);
                let res = self
                    .call_action_base_node(&action, &step.return_name, &step_result, endpoint)
                    .await;
                if res.is_ok() {
                    debug!("endpoint {:?} success return: {:?}", endpoint, res);
                    return res;
                } else {
                    debug!("endpoint {:?} error: {:?}", endpoint, res);
                }
            }
            debug!("report call_action: {:?}", report);
            report
        } else {
            // Calling check node only runs once
            self.call_action_check_node(&action, component, &step.return_name, &step_result)
                .await
        }
    }

    async fn call_action_check_node(
        &self,
        action: &ActionCall,
        node: &ComponentInfo,
        return_name: &String,
        step_result: &StepResult,
    ) -> Result<ActionResponse, anyhow::Error> {
        // prepare rpc call
        let node_url = node.get_url();
        let url = &node_url;

        let client_builder = reqwest::ClientBuilder::new();
        let client = client_builder.danger_accept_invalid_certs(true).build()?;
        // Replace body for transport result of previous step
        let body = Self::replace_string(action.body.clone(), step_result)?;

        debug!("body: {:?}", body);
        let request_builder = client
            .post(url)
            .header("content-type", "application/json")
            .header("x-api-key", node.token.as_str())
            .header("host", node.get_host_header(&self.domain))
            .body(body);

        debug!("request_builder: {:?}", request_builder);

        let sender = request_builder.send();
        pin_mut!(sender);

        // Call rpc
        // Start clock to meansure call time
        let now = Instant::now();
        let res = tokio::time::timeout(
            std::time::Duration::from_secs(action.time_out as u64),
            &mut sender,
        )
        .await;
        //End clock
        let response_time_ms = now.elapsed().as_millis();

        let str_resp = res??.text().await?;
        debug!("response call: {:?}", str_resp);
        self.prepare_result(&str_resp, response_time_ms, action, return_name)
    }

    fn prepare_result(
        &self,
        str_resp: &String,
        response_time_ms: u128,
        action: &ActionCall,
        return_name: &String,
    ) -> Result<ActionResponse, anyhow::Error> {
        let mut str_resp_short = str_resp.clone();
        str_resp_short.truncate(self.config.max_length_report_detail);

        let resp: Value = serde_json::from_str(&str_resp).map_err(|e| {
            anyhow::Error::msg(format!(
                "Err {} when parsing response: {} ",
                e, str_resp_short,
            ))
        })?;

        // get result
        let mut result: HashMap<String, String> = HashMap::new();

        // Add response_time
        result.insert(
            self.config.response_time_key.to_string(),
            response_time_ms.to_string(),
        );

        for (name, path) in action.return_fields.clone() {
            let mut value = resp.clone();
            let path: Vec<String> = path.split("/").map(|s| s.to_string()).collect();
            //debug!("path: {:?}", path);
            for key in path.into_iter() {
                //debug!("key: {:?}", key);
                value = value
                    .get(key.clone())
                    .ok_or(anyhow::Error::msg(format!(
                        "cannot find key {} in result: {:?} ",
                        &key, resp
                    )))?
                    .clone();
                //debug!("value: {:?}", value);
            }
            // Fixme: check other type
            let str_value: String = match value.as_bool() {
                Some(value) => value.to_string(),
                None => value
                    .as_str()
                    .ok_or(anyhow::Error::msg(format!(
                        "value {:?} cannot parse to string",
                        value
                    )))?
                    .to_string(),
            };

            result.insert(name, str_value);
        }

        let action_resp = ActionResponse {
            success: true,
            conclude: CheckMkStatus::Ok,
            return_name: return_name.clone(),
            result,
            message: format!("call {}: {}", return_name.clone(), true),
        };

        debug!("action_resp: {:#?}", action_resp);

        Ok(action_resp)
    }

    async fn call_action_base_node(
        &self,
        action: &ActionCall,
        return_name: &String,
        step_result: &StepResult,
        base_endpoint: &EndpointInfo,
    ) -> Result<ActionResponse, anyhow::Error> {
        let url = &base_endpoint.url;

        let client_builder = reqwest::ClientBuilder::new();
        let client = client_builder.danger_accept_invalid_certs(true).build()?;
        // Replace body for transport result of previous step
        let body = Self::replace_string(action.body.clone(), step_result)?;

        debug!("body call_action_base_node: {:?}", body);
        let request_builder = if base_endpoint.x_api_key.is_empty() {
            client
                .post(url)
                .header("content-type", "application/json")
                .body(body)
        } else {
            client
                .post(url)
                .header("content-type", "application/json")
                .header("x-api-key", base_endpoint.x_api_key.as_str())
                .body(body)
        };

        debug!("request_builder: {:?}", request_builder);

        let sender = request_builder.send();
        pin_mut!(sender);

        // Call rpc
        // Start clock to meansure call time
        let now = Instant::now();
        let res = tokio::time::timeout(
            std::time::Duration::from_secs(action.time_out as u64),
            &mut sender,
        )
        .await;
        //End clock
        let response_time_ms = now.elapsed().as_millis();

        let str_resp = res??.text().await?;

        // Prepare return result
        let res = self.prepare_result(&str_resp, response_time_ms, action, return_name);
        debug!("res: {:?}", res);

        // Check timestamp is reasonable
        match res {
            Ok(mut res) => {
                let create_block_timestamp = res.result.get(&"timestamp".to_string());
                match create_block_timestamp {
                    None => {
                        warn!("There are no timestamp field");
                        Ok(res)
                    }
                    Some(create_block_timestamp) => {
                        // Parse timestamp
                        let create_block_timestamp = u64::from_str_radix(
                            create_block_timestamp.trim_start_matches("0x"),
                            16,
                        )?;
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        let duration_from_last_block = now - create_block_timestamp;
                        debug!(
                            "Time in Seconds from create block {} to now {} last block:{}",
                            create_block_timestamp, now, duration_from_last_block
                        );
                        if duration_from_last_block > 3600 {
                            let message = format!(
                                "The base node is out of sync in {} secs",
                                duration_from_last_block
                            );
                            warn!("{}", message);
                            res.message.push_str(message.as_str());
                            Err(Error::msg(message))
                        } else {
                            Ok(res)
                        }
                    }
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn run_check_steps(
        &self,
        steps: &Vec<CheckStep>,
        component: &ComponentInfo,
    ) -> Result<CheckMkReport, anyhow::Error> {
        let mut step_result: StepResult = HashMap::new();
        let mut status = CheckMkStatus::Ok;
        let mut message = String::new();
        let step_number = steps.len();
        let mut metric: HashMap<String, Value> = HashMap::new();

        for step in steps {
            debug!("step: {:?}", step);
            let report = match step
                .action
                .get("action_type")
                .unwrap()
                .as_str()
                .unwrap_or_default()
            {
                "call" => {
                    let res = self.call_action(component, &step_result, &step).await;
                    debug!("res call_action:{:?}", res);
                    res
                }
                "compare" => {
                    let action: ActionCompare =
                        serde_json::from_value(step.action.clone()).unwrap();
                    debug!("compare action: {:?}", &action);
                    self.compare_action(&action, component, &step.return_name, &step_result)
                }
                _ => Err(anyhow::Error::msg("not support action")),
            };

            info!("report: {:?}", report);

            // Handle report
            match report {
                Ok(report) => {
                    let resp_time = if let Some(response_time_ms) =
                        report.result.get(&*self.config.response_time_key)
                    {
                        Some(response_time_ms.parse::<i64>().unwrap_or_default())
                    } else {
                        None
                    };
                    if let Some(resp_time) = resp_time {
                        let metric_name =
                            format!("{}_{}", report.return_name, self.config.response_time_key);
                        metric.insert(metric_name, resp_time.into());
                    }

                    match report.success {
                        true => {
                            debug!(
                                "Success step: {:?}, report: {:?}",
                                &step.return_name, report
                            );
                            for (key, value) in &report.result {
                                step_result.insert(
                                    format!("{}_{}", &report.return_name, key),
                                    value.clone(),
                                );
                            }
                        }
                        false => {
                            status = step.failed_case.conclude.clone();
                            if step.failed_case.critical {
                                message.push_str(&format!(
                                    "Failed at step {} due to critical error, message: {}",
                                    &step.return_name, report.message
                                ));
                                break;
                            } else {
                                message.push_str(&format!(
                                    "Failed at step {}, message: {}",
                                    &step.return_name, report.message
                                ));
                            }
                        }
                    }
                }
                Err(e) => {
                    message.push_str(&format!(
                        "Failed at step {}, err: {}.",
                        &step.return_name, e
                    ));
                    status = step.failed_case.conclude.clone();
                    if step.failed_case.critical {
                        break;
                    }
                }
            }
        }
        if status == CheckMkStatus::Ok {
            message.push_str(format!("Succeed {} steps. ", step_number).as_str());
        }

        Ok(CheckMkReport {
            status: status as u8,
            service_name: format!(
                "{}-http-{}-{}-{}-{}",
                component.component_type.to_string(),
                component.blockchain,
                component.network,
                component.id,
                component.ip
            ),
            metric: CheckMkMetric { metric },
            status_detail: message,
            success: true,
        })
    }

    fn get_benchmark_url(component: &ComponentInfo) -> String {
        match component.component_type {
            ComponentType::Node => {
                format!("https://{}", component.ip)
            }
            ComponentType::Gateway => {
                format!("https://{}", component.ip)
            }
        }
    }

    pub async fn run_benchmark(
        &self,
        response_time_threshold: f32,
        component: &ComponentInfo,
    ) -> Result<WrkReport, anyhow::Error> {
        let dapi_url = Self::get_benchmark_url(component);
        let mut host = Default::default();
        let mut path = Default::default();
        match component.component_type {
            ComponentType::Node => {
                host = format!("{}.node.mbr.{}", component.id, self.domain);
                path = "/".to_string();
            }
            ComponentType::Gateway => {
                host = format!("{}.gw.mbr.{}", component.id, self.domain);
                path = "/_test_20k".to_string();
            }
        };

        let mut benchmark = WrkBenchmark::build(
            self.config.benchmark_thread,
            self.config.benchmark_connection,
            self.config.benchmark_duration.to_string(),
            self.config.benchmark_rate,
            dapi_url,
            component.token.clone(),
            host,
            self.config.benchmark_script.to_string(),
            self.config.benchmark_wrk_path.to_string(),
            BENCHMARK_WRK_PATH.clone().to_string(),
            response_time_threshold,
        );
        benchmark.run(
            &component.component_type.to_string(),
            &path,
            &component.blockchain,
        )
    }
}
#[derive(Default)]
pub struct GeneratorBuilder {
    inner: CheckComponent,
}

impl GeneratorBuilder {
    pub fn with_domain(mut self, path: String) -> Self {
        self.inner.domain = path;
        self
    }
    pub fn with_base_endpoint(
        mut self,
        base_nodes: HashMap<BlockChainType, HashMap<NetworkType, Vec<EndpointInfo>>>,
    ) -> Self {
        self.inner.base_nodes = base_nodes;
        self
    }
    pub fn with_config(mut self, config: Config) -> Self {
        self.inner.config = config;
        self
    }
    pub fn build(self) -> CheckComponent {
        self.inner
    }
}
