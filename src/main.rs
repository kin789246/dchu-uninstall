pub mod app;
pub mod error;
pub mod exec_cmd;
pub mod inf_metadata;
pub mod logger;
pub mod options;
  
use std::error::Error;
use app::App;
use win32rs::set_dpiawareness_v2;

fn main() -> Result<(), Box<dyn Error>> {
    set_dpiawareness_v2();
    let mut dchu_uninst = App::new();
    dchu_uninst.run()
}