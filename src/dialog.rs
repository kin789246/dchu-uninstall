use windows::core::*;
use windows::Win32::{
    Foundation::*,
    UI::{
        WindowsAndMessaging::*,
        Shell::*,
        Shell::Common::*,
    },
    System::Com::*,
};

use crate::*;
use win_str::*;

pub fn pop_yesno<T>(hwnd: T, msg: &HSTRING) -> MESSAGEBOX_RESULT
where T: Param<HWND> {
    unsafe {
        MessageBoxW(
            hwnd,
            hstr_to_pcwstr(msg),
            w!("Question"),
            MB_YESNO | MB_ICONQUESTION,
        )
    }
}

pub fn pop_info<T>(hwnd: T, msg: &HSTRING) -> MESSAGEBOX_RESULT 
where T: Param<HWND> {
    unsafe {
        MessageBoxW(
            hwnd,
            hstr_to_pcwstr(msg),
            w!("Information"),
            MB_OK | MB_ICONINFORMATION,
        )
    }
}

pub fn pop_error<T>(hwnd: T, msg: &HSTRING) -> MESSAGEBOX_RESULT 
where T: Param<HWND> {
    unsafe {
        MessageBoxW(
            hwnd,
            hstr_to_pcwstr(msg),
            w!("Information"),
            MB_OK | MB_ICONERROR,
        )
    }
}

pub fn file_open() -> Result<()> {
    unsafe {
        CoIncrementMTAUsage()?;
        let dialog: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_ALL)?;

        dialog.SetFileTypes(&[
            COMDLG_FILTERSPEC {
                pszName: w!("Text files"),
                pszSpec: w!("*.txt"),
            },
            COMDLG_FILTERSPEC {
                pszName: w!("All files"),
                pszSpec: w!("*.*"),
            },
        ])?;

        if dialog.Show(None).is_ok() {
            let result = dialog.GetResult()?;
            let path = result.GetDisplayName(SIGDN_FILESYSPATH)?;
            let msg = format!("user picked: {}", path.display());
            pop_info(None, &str_to_hstring(&msg));
            CoTaskMemFree(Some(path.0 as _));
        } else {
            pop_info(None, &str_to_hstring("user canceled"));
        }

        Ok(())
    }
}