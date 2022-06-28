use crate::Timestamp;
use log::debug;
use regex::{Error, Regex};

pub fn get_current_time() -> Timestamp {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("Unix time doesn't go backwards; qed")
        .as_millis() as Timestamp
}
pub fn remove_break_line(input: &String) -> String {
    match Regex::new(r#"^\s*(?P<result>[\s\S]*?)\s*$"#).map(|regex| {
        let caps = regex.captures(input).unwrap();
        caps.name("result").unwrap().as_str().to_string()
    }) {
        Ok(res) => {
            debug!("Ok {}", &res);
            res
        }
        Err(err) => {
            debug!("{}", err);
            input.clone()
        }
    }
}

#[test]
fn test_remove_break_line() {
    let input = "  1\t\n".to_string();
    let output = remove_break_line(&input);
    assert_eq!(output.as_str(), "1");
    assert_eq!(remove_break_line(&String::from(" 12 \n")).as_str(), "12")
}
