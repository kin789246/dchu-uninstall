pub mod app;
pub mod dialog;
pub mod error;
pub mod exec_cmd;
pub mod inf_metadata;
pub mod logger;
pub mod options;
pub mod thread_safe;
pub mod win_str;
pub mod window;
  
use std::error::Error;
use app::App;
use options::Options;

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Options::parse();
    let mut dchu_uninst = App::new(&opts);
    match opts.gui_mode {
        true => {
            App::run(dchu_uninst);
            return Ok(())
        },
        false => {
            return dchu_uninst.proceed();
        } 
    }
}

#[test]
fn remove_dsp() {
    use crate::exec_cmd::*;
    let _intcaudio = "INTELAUDIO";
    let cmd = "get-pnpdevice | \
        where-object { $_.name -like '*High Definition*' } | \
        select-object -property instanceid";
    match ps(cmd) {
        Ok(s) => {
            let mut r = String::new();
            for line in s.lines() {
                if line.contains("HDAUDIO") {
                    r = line.to_owned();
                }
            }
            if !r.is_empty() {
                println!("{r}");
            }
        },
        Err(e) => println!("{}", &e)
    }
}