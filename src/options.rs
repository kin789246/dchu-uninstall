#[derive(Debug)]
pub struct Options {
    pub save_log: bool,
    pub gui_mode: bool,
    pub inf_list: String
}

impl Options {
    pub fn parse() -> Self {
        let mut save_log = false;
        let gui_mode = match std::env::args().len() {
            1 => true,
            _ => false
        };
        let mut inf_list = String::new();
        for line in std::env::args() {
            if line == "-l" {
                save_log = true;
            }
            // match path
            if line.ends_with(".txt") {
                inf_list = line;
            }
        }
        Self { save_log, gui_mode, inf_list }
    }
}