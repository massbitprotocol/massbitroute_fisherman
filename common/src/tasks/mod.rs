pub mod dot;
pub mod eth;
pub mod executor;
pub mod generator;
pub mod ping;

pub use executor::get_eth_executors;
pub use generator::get_eth_task_genrators;
use log::error;
use serde::de::DeserializeOwned;

pub trait LoadConfig<T: DeserializeOwned + Default> {
    fn load_config(path: &str) -> T {
        let json = std::fs::read_to_string(path);
        let config = match json {
            Ok(json) => serde_json::from_str::<T>(&*json).unwrap(),
            Err(err) => {
                error!("Unable to read config file `{}`: {}", path, err);
                Default::default()
            }
        };
        config
    }
}
