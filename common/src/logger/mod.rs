pub mod helper;

/**
 *** The file is to setup logger to either:
 *** - write to file
 *** - output to console
 *** The default option if RUST_LOG is not specified is INFO logging
 **/
use crate::logger::helper::{log_to_console, log_to_file, message};
use lazy_static::lazy_static;
use rand::{random, Rng};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

static GLOBAL_INIT_LOGGING: AtomicBool = AtomicBool::new(false);

lazy_static! {
    static ref RUST_LOG: String = env::var("RUST_LOG").unwrap_or(String::from("info")); // If not specified, assume logging level is INFO
    static ref RUST_LOG_TYPE: String = env::var("RUST_LOG_TYPE").unwrap_or(String::from("console")); // If not specified, assume we're logging to console
}

pub fn init_logger(file_name: &String) -> String {
    sleep(Duration::from_millis(rand::thread_rng().gen_range(0..100)));
    if !GLOBAL_INIT_LOGGING.load(Ordering::Relaxed) {
        GLOBAL_INIT_LOGGING.store(true, Ordering::Relaxed);
    } else {
        return "Logging already inited".to_string();
    }

    /* Logging to file */
    if RUST_LOG_TYPE.to_lowercase().as_str() == "file" {
        log_to_file(file_name, &RUST_LOG);
        return message(&RUST_LOG_TYPE, &RUST_LOG);
    }

    /* Logging to console */
    if RUST_LOG_TYPE.to_lowercase().as_str() == "console" {
        log_to_console(&RUST_LOG);
        return message(&RUST_LOG_TYPE, &RUST_LOG);
    }

    return message(&Default::default(), &RUST_LOG); /* Not logging to anything. This should not reach */
}
