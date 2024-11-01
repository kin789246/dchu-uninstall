use std::fs::OpenOptions;
use std::io::Write;
pub struct Logger;

impl Logger {
    pub fn log(content: &str, path: &str) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(path)?;
        writeln!(file, "{}", &content)?;
        Ok(())
    }
}
