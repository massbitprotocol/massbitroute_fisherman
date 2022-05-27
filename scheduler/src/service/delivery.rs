use std::thread::sleep;
use std::time::Duration;

#[derive(Default)]
pub struct JobDelivery {
    wo
}

impl JobDelivery {
    pub fn new() -> Self {
        JobDelivery {}
    }
    pub fn init(&mut self) {
        loop {
            println!("Delivery jobs");
            sleep(Duration::from_secs(60));
        }
    }
}
