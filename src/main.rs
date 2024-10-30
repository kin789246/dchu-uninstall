pub mod app;
pub mod dialog;
pub mod error;
pub mod exec_cmd;
pub mod inf_metadata;
pub mod logger;
pub mod options;
pub mod win_str;
pub mod window;
  
use std::error::Error;
use app::App;
use options::Options;
use window::Window;
use windows::Win32::UI::HiDpi::*;

fn main() -> Result<(), Box<dyn Error>> {
    set_dpiawareness_v2();
    let opts = Options::parse();
    let mut dchu_uninst = App::new(&opts);
    match opts.gui_mode {
        true => {
            let _ = Window::new(
                &dchu_uninst.get_version(), 
                800, 
                600, 
                dchu_uninst
            ).unwrap();
            return Ok(())
        },
        false => {
            return dchu_uninst.proceed();
        } 
    }
}

fn set_dpiawareness_v2() {
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)
            .unwrap();
    }
}