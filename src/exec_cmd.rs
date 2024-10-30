use std::{borrow::Cow, os::windows::process::CommandExt, process::Command};
use crate::win_str::multi_byte_to_wide_char;
use windows::Win32::Globalization::*;

pub fn ps(c: &str) -> String {
    let op = Command::new("powershell")
        .arg("-command")
        .raw_arg(c)
        .output()
        .expect("Failed to execute command");
    let op_str = String::from_utf8_lossy(&op.stdout).to_string();
    op_str
}

pub fn cmd(c: &str) -> Result<Cow<'static, str>, Cow<'static, str>> {
    let cc = format!("/c chcp 437 && {}", c);
    let op = Command::new("cmd")
        .raw_arg(&cc)
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
        true => Ok(o),
        false => Err(e)
    }
}