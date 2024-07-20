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
    //let drivers_raw = exec_cmd::cmd("pnputil /enum-drivers");
    //let drivers = String::from_utf8_lossy(&drivers_raw);

    log("enum devices with pnputil.exe", &log_name, opts.save_log, true, false);
    //let devices_raw = exec_cmd::cmd("pnputil /enum-devices /relations");
    //let devices = String::from_utf8_lossy(&devices_raw);

    log("parse driver raw list", &log_name, opts.save_log, true, false);
    let mut infs: Vec<InfMetadata> = Vec::new();
    //parse_drivers(&drivers, &devices, &mut infs);
    let drvs = load_txt("..\\..\\drivers.txt")?;
    let devs = load_txt("..\\..\\relations.txt")?;
    parse_drivers(&drvs, &devs, &mut infs);

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
        let inf_list = &load_txt(&opts.inf_list).unwrap();
        uninstall_force(inf_list, &infs, &log_name, opts.save_log);
    }
    log("\n### end log ###", &log_name, opts.save_log, false, false);
    Ok(())
}

fn parse_drivers(drvs: &str, devs: &str, infs: &mut Vec<InfMetadata>) {
    let mut inf: InfMetadata;
    for line in drvs.lines() {
        let two_parts: Vec<_> = line.split(':').collect();
        match two_parts[0] {
            s if s.starts_with("Published Name") => {
                inf= InfMetadata::new();
                inf.published_name = two_parts[1].trim().to_string();
                infs.push(inf);
            },
            s if s.starts_with("Original Name") => {
                let v_inf = infs.last_mut().unwrap();
                v_inf.original_name = two_parts[1].trim().to_string();
            },
            s if s.starts_with("Provider Name") => {
                let v_inf = infs.last_mut().unwrap();
                v_inf.provider_name = two_parts[1].trim().to_string();
            },
            s if s.starts_with("Class Name") => {
                let v_inf = infs.last_mut().unwrap();
                v_inf.class_name = two_parts[1].trim().to_string();
            },
            s if s.starts_with("Class GUID") => {
                let v_inf = infs.last_mut().unwrap();
                v_inf.class_guid = two_parts[1].trim().to_string();
            },
            s if s.starts_with("Driver Version") => {
                let v_inf = infs.last_mut().unwrap();
                v_inf.driver_version = two_parts[1].trim().to_string();
            },
            s if s.starts_with("Signer Name") => {
                let v_inf = infs.last_mut().unwrap();
                v_inf.signer_name = two_parts[1].trim().to_string();
            },
            s if s.starts_with("Extension ID") => {
                let v_inf = infs.last_mut().unwrap();
                v_inf.extension_id = two_parts[1].trim().to_string();
            },
            _ => continue
        }
    }
    let mut iter = devs.lines().into_iter();
    let mut is_line = iter.next();
    while is_line.is_some() {
        let line = is_line.unwrap();
        if line.starts_with("Instance ID") {
            let id = get_value(line);
            let mut des = String::new();
            let mut drv_name = String::new();
            let mut parent = String::new();
            let mut children: Vec<String> = Vec::new();
            let mut ext_infs: Vec<String> = Vec::new();
            is_line = iter.next();
            while is_line.is_some() {
                let sub_line = is_line.unwrap();
                match sub_line {
                    s if s.starts_with("Instance ID") => break,
                    s if s.starts_with("Device Description") => {
                        des = get_value(s);
                        is_line = iter.next();
                    },
                    s if s.starts_with("Driver Name") => {
                        drv_name = get_value(s);
                        is_line = iter.next();
                    },
                    s if s.starts_with("Extension Driver Names") => {
                        ext_infs.push(get_value(s));
                        is_line = iter.next();
                        while is_line.is_some() {
                            let sss = is_line.unwrap();
                            if sss.find(':').is_none() {
                                ext_infs.push(sss.trim().to_string());
                                is_line = iter.next();
                            }
                            else {
                                break;
                            }
                        }
                    },
                    s if s.starts_with("Parent") => {
                        parent = get_value(s);
                        is_line = iter.next();
                    },
                    s if s.starts_with("Children") => {
                        children.push(get_value(s));
                        is_line = iter.next();
                        while is_line.is_some() {
                            let sss = is_line.unwrap();
                            if sss.find(':').is_none() {
                                children.push(sss.trim().to_string());
                                is_line = iter.next();
                            }
                            else {
                                break;
                            }
                        }
                    },
                    s if s.is_empty() => break,
                    _ => {
                        is_line = iter.next();
                    }
                }
            }
            if let Some(inf) = infs
                .iter_mut()
                .find(|ii| ii.published_name.eq_ignore_ascii_case(&drv_name)
            ) {
                inf.instance_id = id.clone();
                inf.device_description = des.clone();
                inf.parent = parent.clone();
                inf.extension_driver_names = ext_infs.clone();
                inf.children = children.clone();
            }
        }
        else {
            is_line = iter.next();
        }
    }
}

fn get_value(line: &str) -> String {
    if let Some(i) = line.find(':') {
        return line[i+1..].trim().to_string();
    }
    else {
        return String::from(line);
    }
}

// pnputil.exe /delete-driver oemNumber /uninstall /force
fn uninstall_force(
    list: &str, 
    infs: &Vec<InfMetadata>, 
    log_path: &str, 
    save_file: bool
) {
    for to_uninstall in list.lines() {
        if infs
            .iter()
            .any(|s| s.original_name.eq_ignore_ascii_case(to_uninstall.trim())
        ) {
            log(
                &format!("uninstall {}", to_uninstall),
                log_path, 
                save_file, 
                true, 
                false
            );
        }
    }
}

fn load_txt(path: &str) -> Result<String, Error> {
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