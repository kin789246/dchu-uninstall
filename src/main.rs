pub mod logger;
pub mod exec_cmd;
pub mod options;
pub mod inf_metadata;

use inf_metadata::InfMetadata;
use logger::Logger;
use options::Options;
use std::{
    collections::{HashMap, VecDeque}, fs::File, io::{Error, Read}
};
use chrono::{self, Local};
fn main() -> Result<(), std::io::Error> {
    let opts = Options::parse();
    let version = env!("CARGO_PKG_VERSION");
    let time_stamp = Local::now().format("%Y-%m%d_%H%M%S").to_string();
    let log_name = format!("{}\\dchu-uninstall_{}.log", &opts.work_dir, &time_stamp);
    println!("{}", &log_name);
    println!("dchu-uninstall {} by Kin|Jiaching", version);
    log(
        &format!("dchu-uninstall {} by Kin|Jiaching\n", version), 
        &log_name, 
        opts.save_log, 
        false, 
        false
    );

    log("enum drivers with pnputil.exe", &log_name, opts.save_log, true, false);
    let drivers = exec_cmd::cmd("pnputil /enum-drivers");

    log("enum devices with pnputil.exe", &log_name, opts.save_log, true, false);
    let devices = exec_cmd::cmd("pnputil /enum-devices /relations");

    log("parse driver raw list", &log_name, opts.save_log, true, false);
    let mut infs: Vec<InfMetadata> = Vec::new();
    parse_drivers(&drivers, &devices, &mut infs);
    // let drvs = load_txt("drivers-mtl-h.txt")?;
    // let devs = load_txt("relations-mtl-h.txt")?;
    // parse_drivers(&drvs, &devs, &mut infs);

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
        on_uninstall(inf_list, &infs, &log_name, opts.save_log, opts.force);
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
                                if !sss.trim().is_empty() {
                                    children.push(sss.trim().to_string());
                                }
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
                inf.instance_id = id;
                inf.device_description = des;
                inf.parent = parent;
                inf.extension_driver_names = ext_infs;
                inf.children = children;
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

fn on_uninstall(
    list: &str, 
    infs: &[InfMetadata], 
    log_path: &str, 
    save_file: bool,
    force: bool
) {
    let mut base_swcs: Vec<InfMetadata> = Vec::new();
    let mut exts: Vec<InfMetadata> = Vec::new();
    for to_uninstall in list.lines() {
        if let Some(oem) = 
            infs
                .iter()
                .find(|inf| 
                    inf.original_name.eq_ignore_ascii_case(to_uninstall.trim()))
        {
            if oem.class_name.eq_ignore_ascii_case("Extension") {
                exts.push(oem.clone());
            }
            else {
                base_swcs.push(oem.clone());
            }
        }
    }

    proceed_uninstall(&base_swcs, &exts, log_path, save_file, force);
}

fn proceed_uninstall(
    infs: &[InfMetadata], 
    exts: &[InfMetadata],
    log_path: &str, 
    save_file: bool,
    force: bool
) {
    let to_proceed: Vec<String> = list_publish_names(&infs);

    // pnputil.exe /delete-driver oemNumber /uninstall /force
    for (i, inf) in to_proceed.iter().enumerate() {
        let org = infs.iter().find(|f| f.published_name == *inf).unwrap();
        let pa = match infs
            .iter()
            .find(|f| !org.parent.is_empty() && f.instance_id.eq(&org.parent))
        {
            Some(s) => s.original_name.clone(),
            None => "none".to_string()
        };
        log(
            &format!("{}. uninstall {}={} of {} parent={}",
                i, inf, org.original_name, org.class_name, pa.clone()), 
            log_path, 
            save_file, 
            true, 
            true
        );
        if force {
            let c = "pnputil /delete-driver ".to_string() + inf + " /uninstall";
            let res = exec_cmd::cmd(&c);
            log(&res, log_path, save_file, false, true);
        }
    }

    for (i, inf) in exts.iter().enumerate() {
        log(
            &format!( "{}. uninstall {}={} of {}", 
                i, inf.published_name, inf.original_name, inf.class_name), 
            log_path, 
            save_file,
            true,
            true
        );
        if force {
            let c = "pnputil /delete-driver ".to_string() 
                + &inf.published_name
                + " /uninstall";
            let res = exec_cmd::cmd(&c);
            log(&res, log_path, save_file, false, true);
        }
    }
}

fn list_publish_names(metadata_list: &[InfMetadata]) -> Vec<String> {
    let mut level_map: HashMap<String, i32> = HashMap::new();
    let mut instance_id_map: HashMap<String, InfMetadata> = HashMap::new();
    let mut queue: VecDeque<InfMetadata> = VecDeque::new();

    for metadata in metadata_list.iter() {
        level_map.insert(metadata.published_name.clone(), 0);
        if !metadata.instance_id.is_empty() {
            instance_id_map.insert(metadata.instance_id.clone(), metadata.clone());
        }
    }

    for metadata in metadata_list.iter() {
        queue.push_back(metadata.clone());
        while !queue.is_empty() {
            let curr_opt = queue.pop_front();
            let curr = curr_opt.unwrap();
            if curr.children.len() > 0 {
                for child in curr.children {
                    if instance_id_map.contains_key(&child) {
                        let cc = instance_id_map.get(&child).unwrap();
                        level_map.entry(cc.published_name.clone()).and_modify(|f| *f += 1);        
                        queue.push_back(cc.clone());
                    }
                }
            }
        }
    }

    let mut ordered_list: Vec<_> = level_map.iter().collect(); 
    ordered_list.sort_by(|a, b| b.1.cmp(a.1));

    ordered_list
        .iter()
        .map(|f| f.0.to_string())
        .collect()
}

fn load_txt(path: &str) -> Result<String, Error> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

fn log(content: &str, path: &str, save_file: bool, add_time: bool, on_screen: bool) {
    if on_screen {
        let mut r: String = String::from(content);
        if add_time {
            let time_stamp = Local::now().format("%Y-%m%d_%H:%M:%S").to_string();
            r = time_stamp + ": " + &r;
        }
        println!("{}", r);
    }
    if save_file {
        Logger::log(content, path, add_time).expect("Log to file failed.");
    }
}