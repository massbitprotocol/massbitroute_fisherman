use crate::models::providers::ProviderStorage;
use common::component::ComponentInfo;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

pub struct ProviderScanner {
    providers: Arc<ProviderStorage>,
}
/*
 * Scan portal and call api to every worker to update latest status
 */
impl ProviderScanner {
    pub fn new(providers: Arc<ProviderStorage>) -> Self {
        ProviderScanner { providers }
    }
    pub fn init(&mut self) {
        loop {
            println!("Get new provider");
            sleep(Duration::from_secs(60));
        }
    }
}
