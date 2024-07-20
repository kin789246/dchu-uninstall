#[derive(Debug)]
pub struct Options {
    pub save_log: bool,
    pub gui_mode: bool,
    pub force: bool,
    pub inf_list: String
}

impl Options {
    pub fn parse() -> Self {
        let mut save_log = false;
        let mut force = false;
        let gui_mode = match std::env::args().len() {
            1 => true,
            _ => false
        };
        let mut inf_list = String::new();
        for line in std::env::args() {
            match &line {
                s if s.eq_ignore_ascii_case("-l") => save_log = true,
                s if s.contains(".txt") => inf_list = line,
                s if s.eq_ignore_ascii_case("-f") => force = true,
                _ => continue
            }
        }
        Self { save_log, gui_mode, force, inf_list }
    }
}