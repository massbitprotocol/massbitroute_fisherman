/**
 *** The file is to setup logger to either:
 *** - write to file
 *** - output to console
 *** The default option if RUST_LOG is not specified is INFO logging
 **/
use crate::helper::{log_to_console, log_to_file, message};
use lazy_static::lazy_static;
use log::LevelFilter;
use std::env;
use std::str::FromStr;

lazy_static! {
    static ref RUST_LOG: String = env::var("RUST_LOG").unwrap_or_else(|_| String::from("info")); // If not specified, assume logging level is INFO
    static ref RUST_LOG_TYPE: String = env::var("RUST_LOG_TYPE").unwrap_or_else(|_| String::from("console")); // If not specified, assume we're logging to console
}

pub fn init_logger(file_name: &str, log_config: Option<&str>) -> String {
    let config = log4rs::load_config_file(log_config.unwrap_or_default(), Default::default());

    /* Logging to file */
    if RUST_LOG_TYPE.to_lowercase().as_str() == "file" {
        log_to_file(file_name, &RUST_LOG);
        return message(&RUST_LOG_TYPE, &RUST_LOG);
    }

    /* Logging to console */
    if RUST_LOG_TYPE.to_lowercase().as_str() == "console" {
        match config {
            Ok(mut config) => {
                println!("Use log config in log.yaml");
                let root = config.root_mut();
                root.set_level(LevelFilter::from_str(&RUST_LOG).expect("Wrong RUST_LOG level"));
                log4rs::init_config(config).expect("Cannot init log config");
            }
            Err(err) => {
                println!(
                    "There is error when loading log.yaml:{:?}, use default log config",
                    err
                );
                log_to_console(&RUST_LOG);
            }
        }
        return message(&RUST_LOG_TYPE, &RUST_LOG);
    }

    message(&Default::default(), &RUST_LOG) /* Not logging to anything. This should not reach */
}
