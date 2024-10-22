use std::{os::windows::process::CommandExt, process::Command};

pub fn ps(c: &str) -> String {
    let op = Command::new("powershell")
        .arg("-command")
        .raw_arg(c)
        .output()
        .expect("Failed to execute command");
    let op_str = String::from_utf8_lossy(&op.stdout).to_string();
    op_str
}

pub fn cmd(c: &str) -> String {
    let op = Command::new("cmd")
        .arg("/c")
        .arg("chcp 437 &&")
        .raw_arg(c)
        .output()
        .expect("Failed to execute command");
    let op_str = String::from_utf8_lossy(&op.stdout).to_string();
    op_str
}