use crate::Timestamp;

pub fn get_current_time() -> Timestamp {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("Unix time doesn't go backwards; qed")
        .as_millis() as Timestamp
}
