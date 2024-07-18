use std::process::Command;

pub fn ps(cmd: &str) -> Vec<u8> {
    let output = Command::new("powershell")
        .arg("-command")
        .arg(cmd)
        .output()
        .expect("Failed to execute command");
    output.stdout
}

pub fn cmd(c: &str) -> Vec<u8> {
    let output = Command::new("cmd")
        .arg("/c")
        .arg(c)
        .output()
        .expect("Failed to execute command");
    output.stdout
}