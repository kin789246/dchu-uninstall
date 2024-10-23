use chrono::prelude::*;
use std::fs::OpenOptions;
use std::io::Write;
pub struct Logger;

impl Logger {
    pub fn log(content: &str, path: &str, add_time: bool) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(path)?;
        let mut result: String = String::from(content);
        if add_time {
            let time = Local::now();
            let time_stamp = time.format("%Y-%m-%d_%H:%M:%S ").to_string();
            result = time_stamp + &result;
        }
        writeln!(file, "{}", result)?;
        Ok(())
    }
}
