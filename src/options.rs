#[derive(Debug)]
pub struct Options {
    pub save_log: bool,
    pub gui_mode: bool,
    pub force: bool,
    pub print_help: bool,
    pub inf_list: String,
    pub work_dir: String
}

impl Options {
    pub fn parse() -> Self {
        let mut save_log = false;
        let mut force = false;
        let mut print_help = false;
        let gui_mode = match std::env::args().len() {
            1 => true,
            _ => false
        };
        let mut inf_list = String::new();
        for line in std::env::args() {
            match &line {
                s if s.eq_ignore_ascii_case("-s") => save_log = true,
                s if s.contains(".txt") => inf_list = line,
                s if s.eq_ignore_ascii_case("-f") => force = true,
                _ => print_help = true
            }
        }
        let work_dir = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        Self { save_log, gui_mode, force, print_help, inf_list, work_dir }
    }
}