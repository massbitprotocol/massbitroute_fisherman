use clap::{App, Arg};
use futures_util::future::join;
use core::logger::init_logger;
use scheduler::server_builder::ServerBuilder;
use log::{debug, info, warn};
use scheduler::SCHEDULER_ENDPOINT;
use scheduler::server_config::AccessControl;
use scheduler::service::http::HttpServiceBuilder;

#[tokio::main]
async fn main() {
    // Load env file
    dotenv::dotenv().ok();

    let _res = init_logger(&String::from("Fisherman Scheduler"));
    let matches = App::new("scheduler")
        .version("0.1")
        .about("fisherman-scheduler")
        .subcommand(create_scheduler_app())
        .get_matches();
    let socket_addr = SCHEDULER_ENDPOINT.as_str();
    let http_service = HttpServiceBuilder::default().build();
    let access_control = AccessControl::default();
    let server = ServerBuilder::default()
        .with_entry_point(socket_addr)
        .with_access_control(access_control)
        .build(http_service);
    info!("Run service ");

    let task_serve = server.serve();
    task_serve.await;
    //join(task_job, task_serve).await;
}

fn create_scheduler_app() -> App<'static> {
    App::new("check-kind")
        .about("check node kind is correct")
        .arg(
            Arg::new("list-node-id-file")
                .short('n')
                .long("list-node-id-file")
                .value_name("list-node-id-file")
                .help("Input list-node-id file")
                .takes_value(true),
        )
        .arg(
            Arg::new("list-gateway-id-file")
                .short('g')
                .long("list-gateway-id-file")
                .value_name("list-gateway-id-file")
                .help("Input list-gateway-id file")
                .takes_value(true),
        )
        .arg(
            Arg::new("list-dapi-id-file")
                .short('d')
                .long("list-dapi-id-file")
                .value_name("list-dapi-id-file")
                .help("Input list-dapi-id file")
                .takes_value(true),
        )
        .arg(
            Arg::new("list-user-file")
                .short('u')
                .long("list-user-file")
                .value_name("list-user-file")
                .help("Input list-user file")
                .takes_value(true),
        )
        .arg(
            Arg::new("check-flow")
                .short('c')
                .long("check-flow")
                .value_name("check-flow")
                .help("Input check-flow file")
                .takes_value(true),
        )
        .arg(
            Arg::new("base-endpoint")
                .short('b')
                .long("base-endpoint")
                .value_name("base-endpoint")
                .help("Input base-endpoint file")
                .takes_value(true),
        )
        .arg(
            Arg::new("massbit-chain-endpoint")
                .short('m')
                .long("massbit-chain-endpoint")
                .value_name("massbit-chain-endpoint")
                .help("Input massbit-chain-endpoint")
                .takes_value(true),
        )
        .arg(
            Arg::new("signer-phrase")
                .short('s')
                .long("signer-phrase")
                .value_name("signer-phrase")
                .help("Input signer-phrase")
                .takes_value(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("output")
                .help("Output file")
                .takes_value(true),
        )
        .arg(
            Arg::new("domain")
                .long("domain")
                .value_name("domain")
                .help("domain name")
                .takes_value(true),
        )
}