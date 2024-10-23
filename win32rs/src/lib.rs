pub mod window;
pub mod dialog;
pub mod win_str;
pub mod control;

use windows::Win32::UI::HiDpi::*;
pub use windows::Win32::Globalization::*;

pub fn set_dpiawareness_v2() {
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use windows::core::*;
    use crate::{dialog, set_dpiawareness_v2, window::Wndrs};

    #[test]
    fn create_wnd() -> Result<()> {
        set_dpiawareness_v2();
        if let Ok(mut wnd) = Wndrs::new("中文測試") {
            wnd.build().unwrap();
        }
        Ok(())
    }

    #[test]
    fn file_open() {
        set_dpiawareness_v2();
        dialog::file_open().unwrap();
    }

    #[test]
    fn cmd_cp950() {
        use crate::win_str::multi_byte_to_wide_char;
        use std::process::Command;
        use std::os::windows::process::CommandExt;
        use windows::Win32::Globalization::
        {
            CP_OEMCP,
            MULTI_BYTE_TO_WIDE_CHAR_FLAGS,
        };
        let op = Command::new("cmd.exe")
            .raw_arg("cmd /c chcp 950 & pnputil /enum-drivers")
            .output()
            .expect("Failed to execute command");
        let o = multi_byte_to_wide_char(
            CP_OEMCP, 
            MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0), 
            &op.stdout
        ).map_or_else(|e| e, |o| o);
        let e = multi_byte_to_wide_char(
            CP_OEMCP, 
            MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0), 
            &op.stderr
        ).map_or_else(|e| e, |o| o);
        match !op.stdout.is_empty() {
            true => println!("{}", o),
            false => println!("{}", e)
        }
    }
}
