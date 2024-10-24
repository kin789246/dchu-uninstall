use std::{
    collections::{HashMap, VecDeque}, 
    error::Error, 
    fs::File, 
    io::Read
};
use chrono::Local;
use crate::{
    error::{ AppError, Kind }, 
    exec_cmd::cmd, 
    inf_metadata::InfMetadata,
    logger::Logger, 
    options::Options
};
use win32rs::{
    dialog::*,
    win_str::*,
};

#[derive(Default)]
pub struct App {
    opts: Options,
    version: String,
    app_log: String,
    drivers: String,
    devices: String,
    infs: Vec<InfMetadata>,
}

impl App {
    const HELP_STR: &'static str = 
        "parameters:\n\
        *.txt [inf list file]\n\
        -v [save logs to file]\n\
        -f [execute pnputil to delete inf]";

    pub fn new() -> Self { 
        let opts = Options::parse();
        let time_stamp = Local::now().format("%Y-%m%d_%H%M%S").to_string();
        let app_log = format!("{}\\dchu-uninstall_{}.log", &opts.work_dir, &time_stamp);
        let version = format!(
            "{} {} by Kin|Jiaching", 
            env!("CARGO_PKG_NAME"), 
            env!("CARGO_PKG_VERSION")
        );
        Self { opts, version, app_log, ..Default::default() }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.log(&self.version, self.opts.save_log, false, true);

        if self.opts.print_help {
            println!("{}", Self::HELP_STR);
            return Err(Box::new(AppError::new(Kind::InvalidFlags)));
        }

        self.log("enum drivers with pnputil.exe", self.opts.save_log, true, false);
        if let Ok(r) = cmd("pnputil /enum-drivers") {
            self.drivers = r.to_string();
        }

        self.log("enum devices with pnputil.exe", self.opts.save_log, true, false);
        if let Ok(r) = cmd("pnputil /enum-devices /relations").to_owned() {
            self.devices = r.to_string();
        }

        self.log("parse driver raw list", self.opts.save_log, true, false);
        self.parse_drivers();
        
        // debug
        // self.drivers = Self::load_txt(r"C:\Users\iec130248\source\dchu-uninstall\template\drivers-mtl-h.txt")?;
        // self.devices = Self::load_txt(r"C:\Users\iec130248\source\dchu-uninstall\template\relations-mtl-h.txt")?;
        // self.parse_drivers();

        // log installed 3rd part drivers
        for inf in &self.infs {
            self.log(&format!("{:?}", inf), self.opts.save_log, false, false);
        }

        match self.opts.gui_mode {
            true => self.auto_mode(),
            false => self.command_mode()
        };
        self.log("### end log ###", self.opts.save_log, false, false);
        Ok(())
    }

    fn command_mode(&self) {
        if !self.opts.inf_list.is_empty() {
            // remove ome?.inf in inf list
            self.log(
                &format!("load inf files from {}", self.opts.inf_list),
                self.opts.save_log, 
                true, 
                false
            );
            if let Ok(inf_list) = Self::load_txt(&self.opts.inf_list) {
                self.on_uninstall(&inf_list);
            }
        }
    }

    fn auto_mode(&mut self) {
        let msg = format!("{}\n{}", &self.version, &Self::HELP_STR);
        pop_info(None, &str_to_hstring(&msg));
    }

    fn parse_drivers(&mut self) {
        // bypass 1st 2 lines for title Microsoft PnP Utility
        let mut iter = self.drivers.lines().skip(2);
        while let Some(line) = iter.next() {
            if line.starts_with("Published Name") {
                let mut inf = InfMetadata::new();
                inf.published_name = Self::get_value(line);
                while let Some(line) = iter.next() {
                    match line {
                        s if s.starts_with("Original Name") => {
                            inf.original_name = Self::get_value(s);
                        },
                        s if s.starts_with("Provider Name") => {
                            inf.provider_name = Self::get_value(s);
                        },
                        s if s.starts_with("Class Name") => {
                            inf.class_name = Self::get_value(s);
                        },
                        s if s.starts_with("Class GUID") => {
                            inf.class_guid = Self::get_value(s);
                        },
                        s if s.starts_with("Driver Version") => {
                            inf.driver_version = Self::get_value(s);
                        },
                        s if s.starts_with("Signer Name") => {
                            inf.signer_name = Self::get_value(s);
                        },
                        s if s.starts_with("Extension ID") => {
                            inf.extension_id = Self::get_value(s);
                        },
                        s if s.is_empty() => break,
                        _ => continue
                    }
                }
                self.infs.push(inf);
            }
        }
        // bypass 1st 2 lines for title Microsoft PnP Utility
        let mut iter = self.devices.lines().skip(2).peekable();
        while let Some(line) = iter.peek() {
            if line.starts_with("Instance ID") {
                let id = Self::get_value(line);
                let mut des = String::new();
                let mut drv_name = String::new();
                let mut parent = String::new();
                let mut children: Vec<String> = Vec::new();
                let mut ext_infs: Vec<String> = Vec::new();
                iter.next();
                while let Some(sub_line) = iter.peek() {
                    match sub_line {
                        s if s.starts_with("Device Description") => {
                            des = Self::get_value(s);
                            iter.next();
                        },
                        s if s.starts_with("Driver Name") => {
                            drv_name = Self::get_value(s);
                            iter.next();
                        },
                        s if s.starts_with("Extension Driver Names") => {
                            ext_infs.push(Self::get_value(s));
                            iter.next();
                            while let Some(sss) = iter.peek() {
                                if !sss.contains("Parent") {
                                    ext_infs.push(Self::get_value(sss));
                                    iter.next();
                                }
                                else {
                                    break;
                                }
                            }
                        },
                        s if s.starts_with("Parent") => {
                            parent = Self::get_value(s);
                            iter.next();
                        },
                        s if s.starts_with("Children") => {
                            children.push(Self::get_value(s));
                            iter.next();
                            while let Some(sss) = iter.peek() {
                                if !sss.is_empty() {
                                    children.push(Self::get_value(sss));
                                    iter.next();
                                }
                                else {
                                    break;
                                }
                            }
                        },
                        s if s.is_empty() => break,
                        _ => { iter.next(); }
                    }
                }
                // debug list infs
                // self.infs.iter().for_each(|f| self.log(&f.published_name, self.opts.save_log, true, false));

                if let Some(inf) = self.infs
                    .iter_mut()
                    .find(|ii| ii.published_name.eq_ignore_ascii_case(&drv_name)) 
                {
                    inf.instance_id = id;
                    inf.device_description = des;
                    inf.parent = parent;
                    inf.extension_driver_names = ext_infs;
                    inf.children = children;
                }
            }
            iter.next();
        }
    }

    fn get_value(line: &str) -> String {
        if let Some(i) = line.find(':') {
            return line[i+1..].trim().to_owned();
        }
        else {
            return line.trim().to_owned();
        }
    }

    fn on_uninstall(&self, list: &str) {
        let mut base_swcs: Vec<InfMetadata> = Vec::new();
        let mut exts: Vec<InfMetadata> = Vec::new();
        for to_uninstall in list.lines() {
            if let Some(oem) = 
                self.infs
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

        self.proceed_uninstall(&base_swcs, &exts);
    }

    fn proceed_uninstall(
        &self,
        base_swcs: &[InfMetadata], 
        exts: &[InfMetadata],
    ) {
        let to_proceed: Vec<String> = Self::list_publish_names(&base_swcs);
        let mut c = 0usize;
        let mut msg = String::from("\n");
        // pnputil.exe /delete-driver oemNumber /uninstall /force
        for inf in to_proceed.iter() {
            let org = self.infs.iter().find(|f| f.published_name == *inf).unwrap();
            let pa = match self.infs
                .iter()
                .find(|f| !org.parent.is_empty() && f.instance_id.eq(&org.parent))
            {
                Some(s) => s.original_name.clone(),
                None => "none".to_owned()
            };
            c += 1;
            let m = format!(
                "{}. uninstall {}={} of {} parent={}\n",
                c, inf, org.original_name, org.class_name, pa.clone()
            ); 
            msg.push_str(&m);
            if self.opts.force {
                self.log(&m, self.opts.save_log, true, true);
                let c = format!("pnputil /delete-driver {} /uninstall", &inf);
                let res = cmd(&c);
                self.log(res.as_ref().unwrap(), self.opts.save_log, false, true);
            }
        }

        for inf in exts.iter() {
            c += 1;
            let m = format!( 
                "{}. uninstall {}={} of {}\n", 
                c, inf.published_name, inf.original_name, inf.class_name
            );
            msg.push_str(&m);
            if self.opts.force {
                self.log(&m, self.opts.save_log, true, true);
                let c = format!(
                    "pnputil /delete-driver {} /uninstall", 
                    &inf.published_name
                );
                let res = cmd(&c);
                self.log(res.as_ref().unwrap(), self.opts.save_log, false, true);
            }
        }
        // list the results
        self.log(&msg, self.opts.save_log, true, true);
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
            .map(|f| f.0.to_owned())
            .collect()
    }

    fn load_txt(path: &str) -> Result<String, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    }

    fn log(
        &self,
        content: &str, 
        save_file: bool, 
        add_time: bool, 
        on_screen: bool
    ) {
        if on_screen {
            let mut r: String = String::from(content);
            if add_time {
                let time_stamp = Local::now().format("%Y-%m%d_%H:%M:%S").to_string();
                r.insert_str(
                    0, 
                    &format!("{}: ", &time_stamp)
                );
            }
            println!("{}", r);
        }
        if save_file {
            Logger::log(content, &self.app_log, add_time).expect("Log to file failed.");
        }
    }
}