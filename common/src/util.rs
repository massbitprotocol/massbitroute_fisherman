use crate::Timestamp;
use anyhow::anyhow;
use log::debug;
use regex::Regex;
/*
 * Get current timestamp in milliseconds
 */
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

pub fn from_str_radix16(input: &str) -> Result<i64, anyhow::Error> {
    let result = Regex::new(r#"0x(?P<result>\w+)"#)
        .map_err(|err| anyhow!("{:?}", err))
        .and_then(|regex| regex.captures(input).ok_or(anyhow!("Capture not found")))
        .and_then(|caps| caps.name("result").ok_or(anyhow!("Match result not found")))
        .and_then(|m| i64::from_str_radix(m.as_str(), 16).map_err(|err| anyhow!("{:?}", err)));
    result
}
#[test]
fn test_remove_break_line() {
    let input = "  1\t\n".to_string();
    let output = remove_break_line(&input);
    assert_eq!(output.as_str(), "1");
    assert_eq!(remove_break_line(&String::from(" 12 \n")).as_str(), "12")
}

#[test]
fn test_from_str_radix16() {
    let input = "0x12";
    let output = from_str_radix16(input).ok();
    assert_eq!(output, Some(18));
}
