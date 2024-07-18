pub mod logger;
pub mod exec_cmd;
pub mod options;
pub mod inf_metadata;

use inf_metadata::InfMetadata;
use logger::Logger;
use options::Options;
use std::{
    fs::File, 
    io::{Error, Read},
};
use chrono::{self, Local};
fn main() -> Result<(), std::io::Error> {
    let version = env!("CARGO_PKG_VERSION");
    let time_stamp = Local::now().format("%Y-%m%d_%H%M%S").to_string();
    let log_name = "inf-remove_".to_owned() + &time_stamp + ".log";
    let opts = Options::parse();
    println!("dchu-uninstall {} by Kin|Jiaching", version);
    log(
        &format!("dchu-uninstall {} by Kin|Jiaching\n", version), 
        &log_name, 
        opts.save_log, 
        false, 
        false
    );

    log("enum drivers with pnputil.exe", &log_name, opts.save_log, true, false);
    let drivers_raw = exec_cmd::cmd("pnputil /enum-drivers");
    let drivers = String::from_utf8_lossy(&drivers_raw);

    log("enum devices with pnputil.exe", &log_name, opts.save_log, true, false);
    let devices_raw = exec_cmd::cmd("pnputil /enum-devices");
    let devices = String::from_utf8_lossy(&devices_raw);

    log("parse driver raw list", &log_name, opts.save_log, true, false);
    let mut infs: Vec<InfMetadata> = Vec::new();
    parse_drivers(&drivers, &devices, &mut infs);

    for inf in &infs {
        log(&format!("{:?}", inf), &log_name, opts.save_log, false, false);
    }

    if !opts.inf_list.is_empty() {
        // remove ome?.inf in inf list
        log(
            &format!("load inf files from {}", opts.inf_list),
            &log_name, 
            opts.save_log, 
            true, 
            false
        );
        let inf_list = &load_inf_txt(&opts.inf_list).unwrap();
        uninstall_force(inf_list, &infs, &log_name, opts.save_log);
    }
    log("\n### end log ###", &log_name, opts.save_log, false, false);
    Ok(())
}

fn parse_drivers(drvs: &str, _devs: &str, infs: &mut Vec<InfMetadata>) {
    let mut inf: InfMetadata;
    for line in drvs.lines() {
        let two_parts: Vec<_> = line.split(':').collect();
        match two_parts[0] {
            s if s.contains("Published Name") => {
                inf= InfMetadata::new();
                inf.published_name = two_parts[1].trim().to_string();
                infs.push(inf);
            },
            s if s.contains("Original Name") => {
                let v_inf = infs.last_mut().unwrap();
                v_inf.original_name = two_parts[1].trim().to_string();
            },
            s if s.contains("Provider Name") => {
                let v_inf = infs.last_mut().unwrap();
                v_inf.provider_name = two_parts[1].trim().to_string();
            },
            s if s.contains("Class Name") => {
                let v_inf = infs.last_mut().unwrap();
                v_inf.class_name = two_parts[1].trim().to_string();
            },
            s if s.contains("Class GUID") => {
                let v_inf = infs.last_mut().unwrap();
                v_inf.class_guid = two_parts[1].trim().to_string();
            },
            s if s.contains("Driver Version") => {
                let v_inf = infs.last_mut().unwrap();
                v_inf.driver_version = two_parts[1].trim().to_string();
            },
            s if s.contains("Signer Name") => {
                let v_inf = infs.last_mut().unwrap();
                v_inf.signer_name = two_parts[1].trim().to_string();
            },
            s if s.contains("Extension ID") => {
                let v_inf = infs.last_mut().unwrap();
                v_inf.extension_id = two_parts[1].trim().to_string();
            },
            _ => continue
        }
    }
}

// pnputil.exe /delete-driver oemNumber /uninstall /force
fn uninstall_force(list: &str, infs: &Vec<InfMetadata>, log_path: &str, save_file: bool) {
    for to_uninstall in list.lines() {
        if infs.iter().any(|s| s.original_name.eq_ignore_ascii_case(to_uninstall.trim())) {
            log(
                &format!("uninstall {}", to_uninstall),
                log_path, 
                save_file, 
                true, 
                true
            );
        }
    }
}

fn load_inf_txt(path: &str) -> Result<String, Error> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

fn log(content: &str, path: &str, save_file: bool, add_time: bool, on_screen: bool) {
    if on_screen {
        println!("{}", content);
    }
    if save_file {
        Logger::log(content, path, add_time).expect("Log to file failed.");
    }
}