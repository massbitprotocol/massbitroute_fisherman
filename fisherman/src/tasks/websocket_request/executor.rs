use crate::tasks::toanyhowerror;
use crate::tasks::websocket_request::wsclient_builder::WebSocketClientBuilder;
use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::{JobDetail, JobResultDetail};
use common::jobs::{Job, JobResult};
use common::models::ResponseValues;
use common::tasks::executor::TaskExecutor;
use common::tasks::http_request::HttpRequestError;
use common::tasks::websocket_request::{
    JobWebsocket, JobWebsocketResponse, JobWebsocketResponseDetail,
};
use common::util::get_current_time;
use common::WorkerId;
use log::{debug, error, trace};

use tokio::sync::mpsc::Sender;
use websocket::header::Headers;
use websocket::native_tls::TlsConnector;

use websocket::{Message, OwnedMessage};

#[derive(Clone, Debug, Default)]
pub struct WebsocketRequestExecutor {
    _worker_id: WorkerId,
}

impl WebsocketRequestExecutor {
    pub fn new(worker_id: WorkerId) -> Self {
        WebsocketRequestExecutor {
            _worker_id: worker_id,
        }
    }

    pub async fn call_websocket_request(
        &self,
        request: &JobWebsocket,
    ) -> Result<JobWebsocketResponse, HttpRequestError> {
        let mut headers = Headers::new();
        for (key, value) in request.headers.iter() {
            headers.set_raw(key.clone(), vec![value.clone().as_bytes().to_vec()]);
        }
        debug!(
            "call websocket to url {:?} with headers {:?} and body {:?}",
            &request.url, &headers, &request.body
        );
        if let Ok(mut client_builder) = WebSocketClientBuilder::new(request.url.as_str())
            .map_err(toanyhowerror)
            .map(|builder| builder.custom_headers(&headers))
        {
            let message = request
                .body
                .as_ref()
                .map(|body| Message::text(body.to_string()))
                .unwrap_or_else(|| Message::ping(Vec::<u8>::default()));
            let request_timestamp = get_current_time();
            //Try to connect
            let received_message = if request.url.starts_with("wss") {
                let mut tsl_connector_builder = TlsConnector::builder();
                tsl_connector_builder.danger_accept_invalid_hostnames(true);
                let mut client = client_builder
                    .connect_secure(tsl_connector_builder.build().ok())
                    .map_err(|err| {
                        debug!("Cannot connect to {:?}", &err);
                        HttpRequestError::SendError(format!(
                            "Can not connect to {:?}",
                            &request.url
                        ))
                    })?;
                if let Err(err) = client.send_message(&message) {
                    debug!("Error {:?}", &err);
                } else {
                    debug!("Connected");
                }
                let mut received_message = OwnedMessage::Ping(vec![]);
                while let Ok(mess) = client.recv_message() {
                    match mess {
                        val @ OwnedMessage::Text(_) => {
                            received_message = val;
                            break;
                        }
                        OwnedMessage::Binary(_) => {}
                        OwnedMessage::Close(_) => {}
                        OwnedMessage::Ping(_) => {
                            client
                                .send_message(&Message::pong(vec![]))
                                .map_err(|err| HttpRequestError::SendError(format!("{:?}", err)))?;
                        }
                        OwnedMessage::Pong(_) => {}
                    }
                }
                match client.shutdown() {
                    Ok(_) => {}
                    Err(_) => {}
                };
                received_message
            } else {
                let mut client = client_builder.connect_insecure().map_err(|err| {
                    HttpRequestError::SendError(format!(
                        "Can not connect to {:?}, with error: {err:?}",
                        &request.url
                    ))
                })?;
                client
                    .send_message(&message)
                    .map_err(|err| HttpRequestError::SendError(format!("{:?}", err)))?;
                let mut received_message = OwnedMessage::Ping(vec![]);
                while let Ok(mess) = client.recv_message() {
                    match mess {
                        val @ OwnedMessage::Text(_) => {
                            received_message = val;
                            break;
                        }
                        OwnedMessage::Binary(_) => {}
                        OwnedMessage::Close(_) => {}
                        OwnedMessage::Ping(_) => {
                            client
                                .send_message(&Message::pong(Vec::<u8>::default()))
                                .map_err(|err| HttpRequestError::SendError(format!("{:?}", err)))?;
                        }
                        OwnedMessage::Pong(_) => {}
                    }
                }
                match client.shutdown() {
                    Ok(_) => {}
                    Err(_) => {}
                };
                received_message
            };
            let response = if let OwnedMessage::Text(mess) = received_message {
                debug!("Received Socket message from provider {:?}", &request.url);
                let response_timestamp = get_current_time();
                let response_values =
                    ResponseValues::extract_values(mess.as_str(), &request.response_values)
                        .unwrap_or_default();
                Ok(JobWebsocketResponse {
                    request_timestamp,
                    response_duration: response_timestamp - request_timestamp,
                    detail: JobWebsocketResponseDetail::Values(response_values),
                    error_code: 0,
                    message: "".to_string(),
                })
            } else {
                Err(HttpRequestError::GetBodyError(
                    "Invalid response data".to_string(),
                ))
            };

            response
        } else {
            Err(HttpRequestError::BuildError(format!(
                "Cannot connect to {:?}",
                &request.url
            )))
        }
    }
}

#[async_trait]
impl TaskExecutor for WebsocketRequestExecutor {
    async fn execute(&self, job: &Job, result_sender: Sender<JobResult>) -> Result<(), Error> {
        trace!("WebsocketRequestExecutor execute job {:?}", &job);
        if let JobDetail::Websocket(request) = &job.job_detail {
            let res = self.call_websocket_request(request).await;
            let response = match res {
                Ok(res) => {
                    debug!(
                        "Received Response from provider's websocket {:?} {:?}",
                        job.component_type.to_string(),
                        job.component_url
                    );
                    res
                }
                Err(err) => {
                    error!(
                        "Cannot connect to websocket on provider {:?} {:?}",
                        job.component_type.to_string(),
                        job.component_url
                    );
                    JobWebsocketResponse::new_error(
                        get_current_time(),
                        err.get_code(),
                        err.get_message().as_str(),
                    )
                    //err.into()
                }
            };
            trace!("Websocket request result {:?}", &response);
            if let JobDetail::Websocket(request) = &job.job_detail {
                let job_result = JobResult::new(
                    JobResultDetail::Websocket(response),
                    request.chain_info.clone(),
                    job,
                );
                debug!("send job_result: {:?}", job_result);
                let _res = result_sender.send(job_result).await;
            };
        } else {
            debug!("Invalid job detail");
        }
        Ok(())
    }

    fn can_apply(&self, job: &Job) -> bool {
        match &job.job_detail {
            JobDetail::Websocket(request) => {
                if request.url.len() > 0 {
                    return true;
                }
                log::warn!("Missing url for job {:?}", job);
                return false;
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tasks::WebsocketRequestExecutor;
    use common::component::ComponentInfo;

    use common::tasks::websocket_request::{JobWebsocket, JobWebsocketResponseDetail};
    use serde_json::Value;
    #[derive(Debug)]
    struct ProviderInfo {
        url: String,
        id: String,
        api_key: String,
        provider_type: String,
    }
    fn new_executor() -> WebsocketRequestExecutor {
        WebsocketRequestExecutor::new("test_websocket_worker_id".to_string())
    }
    fn new_test_job(job_detail: &str) -> JobWebsocket {
        let _component = ComponentInfo {
            blockchain: "rinkerby".to_string(),
            network: "main".to_string(),
            id: "".to_string(),
            user_id: "".to_string(),
            ip: "".to_string(),
            zone: Default::default(),
            country_code: "".to_string(),
            token: "".to_string(),
            component_type: Default::default(),
            endpoint: None,
            status: "".to_string(),
        };
        let job_websocket: JobWebsocket = serde_json::from_str(job_detail).unwrap();
        job_websocket
    }
    fn set_provider_info(job: &mut JobWebsocket, provider: &ProviderInfo) {
        job.url = provider.url.clone();
        if !provider.id.is_empty() {
            job.headers.insert(
                "Host".to_string(),
                format!(
                    "ws-{}.{}.mbr.massbitroute.net",
                    provider.id, provider.provider_type
                ),
            );
        }
        if provider.api_key.len() > 0 {
            job.headers
                .insert("X-Api-Key".to_string(), provider.api_key.clone());
        }
    }
    const ETH_REQUEST: &str = r#"{
            "url": "",
            "headers": {},
            "chain_info": {"chain": "eth", "network": "mainnet"},
            "body": {
              "jsonrpc": "2.0",
              "method": "eth_getBlockByNumber",
              "params": [
                  "0xa7e964",
                  false
              ],
              "id": 1
            },
            "response_type": "json",
            "response_values": {
                "timestamp": ["result","timestamp"],
                "number": ["result","number"],
                "hash": ["result", "hash"]
            }
        }"#;
    #[ignore]
    #[tokio::test]
    async fn test_eth_gateway_websocket() {
        let executor = new_executor();
        let providers = vec![ProviderInfo {
            url: "wss://34.101.135.131/_node/058a6e94-8b65-46ad-ab52-240a7cb2c36a-ws/".to_string(),
            id: "7bc87cee-1425-4bfd-8f24-ba8f25ee5e2e".to_string(),
            api_key: "cUH7LakFjSfLhrqoISFmNA".to_string(),
            provider_type: "gw".to_string(),
        }];
        let mut job_websocket = new_test_job(ETH_REQUEST);
        for provider in providers.iter() {
            set_provider_info(&mut job_websocket, provider);
            println!("Set provider {:?} to job {:?}", provider, &job_websocket);
            let response = executor
                .call_websocket_request(&job_websocket)
                .await
                .unwrap_or_default();
            match response.detail {
                JobWebsocketResponseDetail::Body(_body) => {
                    println!("Test failed for provider {:?}", provider);
                    assert_eq!(0, 1);
                }
                JobWebsocketResponseDetail::Values(values) => {
                    println!("Test pass for provider {:?}", provider);
                    assert_eq!(
                        values.get("number").map(|val| val.clone()),
                        Some(Value::from("0xa7e964"))
                    )
                }
            }
        }
    }
    // Ignore because the node is turnoff
    #[ignore]
    #[tokio::test]
    async fn test_node_websocket() {
        let executor = new_executor();
        let providers = vec![
            ProviderInfo {
                url: "wss://34.101.146.31/".to_string(),
                id: "058a6e94-8b65-46ad-ab52-240a7cb2c36a".to_string(),
                api_key: "lSP1lFN9I_izEzRi_jBapA".to_string(),
                provider_type: "node".to_string(),
            },
            ProviderInfo {
                url: "wss://34.69.64.125/".to_string(),
                id: "0dc806f2-59b0-4300-b3e5-1e18b3095e10".to_string(),
                api_key: "ZCY7yfAnkt1R7gL_x9kCKw".to_string(),
                provider_type: "node".to_string(),
            },
        ];
        let mut job_websocket = new_test_job(ETH_REQUEST);
        for provider in providers.iter() {
            set_provider_info(&mut job_websocket, provider);
            println!("Set provider {:?} to job {:?}", provider, &job_websocket);
            let response = executor
                .call_websocket_request(&job_websocket)
                .await
                .unwrap_or_default();
            match response.detail {
                JobWebsocketResponseDetail::Body(_body) => {
                    println!("Test failed for provider {:?}", provider);
                    assert_eq!(0, 1);
                }
                JobWebsocketResponseDetail::Values(values) => {
                    println!("Test pass for provider {:?}", provider);
                    assert_eq!(
                        values.get("number").map(|val| val.clone()),
                        Some(Value::from("0xa7e964"))
                    )
                }
            }
        }
    }

    #[tokio::test]
    async fn test_datasource_websocket() {
        let executor = new_executor();
        let urls = vec![
            "ws://34.121.91.195:8546",
            "wss://eth-mainnet.nodereal.io/ws/v1/c7c206714f38461c83c84a6a8448b552",
        ];

        let mut job_websocket = new_test_job(ETH_REQUEST);
        let _result = "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"baseFeePerGas\":\"0xb\",\"difficulty\":\"0x1\",\"extraData\":\"0xd883010a11846765746888676f312e31372e38856c696e75780000000000000062c31fbb4f3bb9c5e5ca7d837e1e169a6f19fcdc46eeb6577b323c8e880f876471c665e64b53066776def2fc0b8df233a4c9c406b3361f83c7eee8110938a79b00\",\"gasLimit\":\"0x1c9c380\",\"gasUsed\":\"0x54c72f\",\"hash\":\"0x67deeccb4eb8230302ed7a526e9a019f00bbf67beb04f520b0f9877e78bf4530\",\"logsBloom\":\"0x8600800002000000208010080000501004000024009005000081000000000202000081000080410080110218000001800400284001280000444000111224000100900c014800000c201801880804000098200100820405011000010006080000018080004382000c00500000024018484e842100800000000000101040804002000008084006340002490203008040108840000880820404300800044000008102000081400020044008c10800000c080c00020000100a24808604000800014000040002424088240000100184006000800034004000000a000c048080a061002050001201000000000020400c009010004000004080204088020002080d8091\",\"miner\":\"0x0000000000000000000000000000000000000000\",\"mixHash\":\"0x0000000000000000000000000000000000000000000000000000000000000000\",\"nonce\":\"0x0000000000000000\",\"number\":\"0xa7e964\",\"parentHash\":\"0x44ccfbb4e3a85b1f49bb66fa431de664af1e953fc4d6e399f2c4828de2aeb2a1\",\"receiptsRoot\":\"0x70da079d67bff1e714fa3235f8fb4f1f91b85f7e5f14dc6b1e47c4f9e6d10506\",\"sha3Uncles\":\"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347\",\"size\":\"0x34b7\",\"stateRoot\":\"0x367ffd56fd6cea699ebe52f4ee36934171406e8dd6e21c101bfdae54067a33e7\",\"timestamp\":\"0x62cbc6f3\",\"totalDifficulty\":\"0x1133af8\",\"transactions\":[\"0x1f1851c843b09c16e2f8649d5f292d1b1d5d0480770db817ef923e8da83a176c\",\"0x67581b697e4174a8ed2bd46e446048176e5d112706ab7597d1f48081bbc9d9a7\",\"0x767e3360e27b8e478f923b99a54ed263f4237ab324b4b7a8c3b8790000c30701\",\"0x9abe80ec862100f4a33475b95f37e36f58ef36e3e627ebdb9428014b6ce89362\",\"0x3aca9e3fe3636696a9da88ceeb79040cb315f3a04efd7b2f386e44a01d0e89d0\",\"0x8bde2514181b114c6727ec9d2854461e0aed813cf85972ff7db877595d313742\",\"0x66e4d6b61ce8f8be57ce43ddd98c3fc9095e2c77ba06677e1b912566e37f4a05\",\"0xaa35ea524ca4796afa0674a3989861fca2fbbd77b1b6e6a9eee225e9eca3cc34\",\"0x0cccd76924162fd3e661a96ef8711826f08ac8a93f76899b015106cefdd71cbc\",\"0xc84d7f96a26ce42428710fd41ded9d514827d0aa93ffc0aacc1d7b93e88dd7fb\",\"0x9e29912222fdd223b939c2d165ac6a193dd8f4c29672f946a53db1abd6ff1081\",\"0xf9cff963ebedb7d13e30a48f5e7dc3296649916dda29dd3d6bd930172230f669\",\"0xc29a3ca9011e22c6907d4851566c8a837e58cb29737ffca1e6cb64253cd40aae\",\"0xdfaed9ffae43f7cbca774b0769c4c5f40470abfe4d8b835530b6ee621d933944\",\"0x9cd74f523ac136afb03c5792d7a9ed3c555c68a72d27df592ccd9913db99dd9b\",\"0x2342e6d0ec4f81085fb2bd6d6ca05a7ac3c430ff569a4a24a3bfabf1c6a6fc11\",\"0x4a7521552f4e81ac1a3ee9caf8fd1f571592751cf9e5c06f13b617487471721f\",\"0xc55d27414d1b875a46acde7a18a639dc6b2b7a223ff8aee2f16598a79d12e801\",\"0xa52bef82d35b7f6b470fc0c1ba752e81680dcc778d3a7f4388551d8154d4d48c\",\"0x0b1d814cd415496959943c588fd9fb0a8184142e5d0fb0cd9a8afaad1757eea0\",\"0xbd99732bdc0264d7474134253615cb417ab70f2aba73f58756abb8a1e475f2f2\",\"0x6402df7e66528d3b3ae494f4630ae05fb2dff024b4eed3b9e72f1310d86357c3\",\"0xa60a7d53f614b6883504818113a1cc9bf65a5da1b5158651e4c8ac123b1a7f8f\",\"0xdc8a1d1f83077c8681bbe8003164c0d28e2a8f3f49d3f6ac7f0deea03113a6ab\",\"0x54008ffab3c06e6d483f3dea2ab49917b404279b4793ff513fcdd424b737fe45\",\"0x903bd51d61d1da330d83fc557f5a3a44b6d30177809bfa44af3e4a407e231243\",\"0xadaf1acee770c3cab7a77d8338370a2818cd963dd59bdad76394829fefe22495\",\"0x29b48bd1791d3be7f8867caf81f7e8e0e3c92f7723095e41b2d710c327fd782f\",\"0x7098fa760601be138166877e1c860322ee49b5486f2d266f5e5d3aab316fd21f\",\"0x915f594a9da92646ebb1c686aee6aeaf94baa397792ac7dadcd7d1dd0a109493\",\"0x045f59df31aa74363895a4726d402be791018bfd48394d121a4d815f6f516013\",\"0xd1fc69dc4c5eb1942fc9d8c0d967c4ae5b25c8212695c8050502441b965148a6\",\"0x4c4dcb691f91ae95446524f5fd01dd130376088214f2c01777bead8ee36c78b9\",\"0x2a4839305c5f50381f1f859cd17e5b6090164eaf67ad70ab3bcca6526dc11378\",\"0xe3c0e516eac7ee206717d8c44ea87da2ebde2bfe836c8b3517c79db1b8643093\"],\"transactionsRoot\":\"0x96f02e2d0c1f7f510a7e8df072f2839628f39377b3f1121f4fc88aa4808526c6\",\"uncles\":[]}}\n";
        for url in urls {
            job_websocket.url = url.to_string();
            let response = executor
                .call_websocket_request(&job_websocket)
                .await
                .unwrap_or_default();
            match response.detail {
                JobWebsocketResponseDetail::Body(_body) => {
                    println!("Test failed for url {:?}", url);
                    assert_eq!(0, 1);
                }
                JobWebsocketResponseDetail::Values(values) => {
                    println!("Test pass for url {:?}", url);
                    assert_eq!(
                        values.get("number").map(|val| val.clone()),
                        Some(Value::from("0xa7e964"))
                    )
                }
            }
        }
    }
}
