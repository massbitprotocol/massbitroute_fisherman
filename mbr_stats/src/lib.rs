// #![feature(fmt_internals)]
// #![feature(print_internals)]

#[path = "chain_adapter.rs"]
pub mod chain_adapter;
pub mod component_stats;

use anyhow::anyhow;

use lazy_static::lazy_static;
use std::env;
use std::str::FromStr;

const DEFAULT_HTTP_REQUEST_TIMEOUT_MS: u64 = 5000;

lazy_static! {
    pub static ref SIGNER_PHRASE: String =
        env::var("STAT_SIGNER_PHRASE").expect("There is no env var STAT_SIGNER_PHRASE");
    pub static ref PORTAL_AUTHORIZATION: String =
        env::var("PORTAL_AUTHORIZATION").expect("There is no env var PORTAL_AUTHORIZATION");
    pub static ref SCHEME: Scheme =
        Scheme::from_str(&env::var("SCHEME").expect("There is no env var SCHEME"))
            .expect("Cannot parse var SCHEME");
    pub static ref URL_CHAIN: String =
        env::var("URL_CHAIN").unwrap_or_else(|_| "ws://chain.massbitroute.net:9944".to_string());
    pub static ref ENVIRONMENT: Environment =
        Environment::from_str(&env::var("ENVIRONMENT").expect("There is no env var ENVIRONMENT"))
            .expect("Cannot parse var ENVIRONMENT");
}

#[derive(Debug, PartialEq, Eq)]
pub enum Scheme {
    Https,
    Http,
}

impl Scheme {
    pub fn to_http_string(&self) -> String {
        match self {
            Scheme::Https => "https".to_string(),
            Scheme::Http => "http".to_string(),
        }
    }
    pub fn to_ws_string(&self) -> String {
        match self {
            Scheme::Https => "wss".to_string(),
            Scheme::Http => "ws".to_string(),
        }
    }
}

impl FromStr for Scheme {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "https" => Ok(Scheme::Https),
            "http" => Ok(Scheme::Http),
            _ => Err(anyhow!("Cannot parse {s} to Schema")),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Environment {
    Local,
    DockerTest,
    Release,
    Production,
}

impl ToString for Environment {
    fn to_string(&self) -> String {
        match self {
            Environment::Local => "local".to_string(),
            Environment::DockerTest => "docker_test".to_string(),
            Environment::Release => "release".to_string(),
            Environment::Production => "production".to_string(),
        }
    }
}

impl FromStr for Environment {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(Environment::Local),
            "docker_test" => Ok(Environment::DockerTest),
            "production" => Ok(Environment::Production),
            _ => Err(anyhow!("Cannot parse {s} to Environment")),
        }
    }
}
