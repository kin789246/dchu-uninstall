use std::{
    collections::{HashMap, VecDeque}, 
    error::Error, 
    fs::{read_dir, File}, 
    io::Read, 
    mem, 
    path::Path, 
    sync::{Arc, Mutex}, 
    thread, 
    time::Duration
};
use chrono::Local;
use crate::{
    dialog::*, 
    error::{ AppError, Kind }, 
    exec_cmd::*, 
    inf_metadata::InfMetadata, 
    logger::Logger, 
    options::Options, 
    thread_safe::ThreadSafeHwnd, 
    win_str::str_to_hstring, 
    window::*
};
use windows::{
    core::HSTRING,
    Win32::{
        Foundation::*,
        UI::WindowsAndMessaging::*,
        System::DataExchange::COPYDATASTRUCT,
    }
};

#[derive(Default, Clone)]
pub struct App {
    opts: Options,
    version: String,
    app_log: String,
    drivers: String,
    devices: String,
    help: String,
    infs_path: String,
    infs: Vec<InfMetadata>,
    window: Option<ThreadSafeHwnd>,
}

impl App {
    pub fn new(opts: &Options) -> Self { 
        let time_stamp = Local::now().format("%Y-%m%d_%H%M%S").to_string();
        let app_log = format!("{}\\dchu-uninstall_{}.log", &opts.work_dir, &time_stamp);
        let infs_path = opts.work_dir.clone();
        let version = format!(
            "{} {} by Kin|Jiaching", 
            env!("CARGO_PKG_NAME"), 
            env!("CARGO_PKG_VERSION")
        );
        let help: String = "parameters:\n\
            *.txt [inf list file]\n\
            -v [save logs to file]\n\
            -f [execute pnputil to delete inf]"
            .to_owned();

        Self { 
            opts: opts.clone(), 
            version, 
            app_log, 
            help,
            infs_path,
            ..Default::default() 
        }
    }

    pub fn run(app: App) {
        let version = app.version.clone();
        Window::new(&version, 800, 600, app).unwrap();
    }

    pub fn set_path(&mut self, path: &str) {
        self.infs_path = path.to_owned();
    }

    pub fn get_infs_path(&self) -> HSTRING {
        HSTRING::from(&self.infs_path)
    }

    pub fn proceed(&mut self) -> Result<(), Box<dyn Error>> {
        self.log_console(&self.version, self.opts.save_log, false, true);

        if self.opts.print_help {
            self.log_console(
                &format!("\n{}", &self.help),
                self.opts.save_log, 
                true, 
                true
            );
            return Err(Box::new(AppError::new(Kind::InvalidFlags)));
        }

        self.log_console("enum drivers with pnputil.exe", self.opts.save_log, true, false);
        if let Ok(r) = cmd("pnputil /enum-drivers") {
            self.drivers = r.to_string();
        }

        self.log_console("enum devices with pnputil.exe", self.opts.save_log, true, false);
        if let Ok(r) = cmd("pnputil /enum-devices /relations").to_owned() {
            self.devices = r.to_string();
        }

        self.log_console("parse driver raw list", self.opts.save_log, true, false);
        self.parse_drivers();
        
        // debug
        // self.drivers = Self::load_txt(r"C:\Users\iec130248\source\dchu-uninstall\template\drivers-mtl-h.txt")?;
        // self.devices = Self::load_txt(r"C:\Users\iec130248\source\dchu-uninstall\template\relations-mtl-h.txt")?;
        // self.parse_drivers();

        // log installed 3rd part drivers
        for inf in &self.infs {
            self.log_console(&format!("{:?}", inf), self.opts.save_log, false, false);
        }
        if !self.opts.inf_list.is_empty() {
            // remove ome?.inf in inf list
            self.log_console(
                &format!("load inf files from {}", self.opts.inf_list),
                self.opts.save_log, 
                true, 
                false
            );
            match Self::load_txt(&self.opts.inf_list) {
                Ok(inf_list) => self.on_uninstall(inf_list.lines()),
                Err(e) => return Err(e)
            }
        }
        self.log_console("### end log ###", self.opts.save_log, false, false);
        Ok(())
    }

    pub fn init_gui(&mut self, hwnd: HWND) {
        self.window = Some(ThreadSafeHwnd(hwnd));
        self.log_gui(&self.version, false, true);
        self.log_gui("\r\nREADY TO GO", false, true);
    }

    pub fn remove_btn_click(app: Arc<Mutex<Self>>) {
        let app = app.clone();
        thread::spawn(move || {
            let mut app = app.lock().unwrap();
            app.post_message(Window::CTRL_EN_DIS, false);
            if app.infs.is_empty() {
                app.load_infs();
            }
            app.log_gui(
                &format!("get inf files list from {}", &app.infs_path), 
                true, 
                true
            );
            match app.get_inf_list() {
                Ok(s) => {
                    app.log_gui(&s, false, true);
                    app.confirm_uninstall();
                    app.on_uninstall(s.lines());
                },
                Err(e) => {
                    app.log_gui(&e, true, true);
                }
            }
            app.post_message(Window::CTRL_EN_DIS, true);
        });
    }

    fn remove_dsp(&self) {
        let intcaudio = "INTELAUDIO";
        let c = "get-pnpdevice | \
            where-object { $_.name -like '*High Definition DSP*' } | \
            select-object -property instanceid";
        if let Ok(s) = ps(c) {
            let mut r = String::new();
            for line in s.lines() {
                if line.contains(intcaudio) {
                    r = line.to_owned();
                    break;
                }
            }
            if !r.is_empty() {
                self.log(
                    "remove INTEL High Definition DSP", 
                    self.opts.save_log, 
                    true, 
                    true
                );
                let c = format!("pnputil /remove-device \"{}\" /subtree", &r);
                if let Ok(s) = cmd(&c) {
                    self.log(&s, self.opts.save_log, true, true);
                }
                if let Ok(s) = cmd("pnputil /scan-devices") {
                    self.log(&s, self.opts.save_log, true, true);
                }
            }
        }
    }

    fn load_infs(&mut self) {
        // self.log_gui("enum drivers with pnputil.exe", true, true);
        // if let Ok(r) = cmd("pnputil /enum-drivers") {
        //     self.drivers = r.to_string();
        // }
        // self.log_gui("enum devices with pnputil.exe", true, true);
        // if let Ok(r) = cmd("pnputil /enum-devices /relations").to_owned() {
        //     self.devices = r.to_string();
        // }

        // debug
        self.drivers = Self::load_txt(r"C:\Users\iec130248\source\dchu-uninstall\template\drivers-mtl-h.txt").unwrap();
        self.devices = Self::load_txt(r"C:\Users\iec130248\source\dchu-uninstall\template\relations-mtl-h.txt").unwrap();

        self.log_gui("parse driver raw list", true, true);
        self.parse_drivers();
        
        // log installed 3rd part drivers
        for inf in &self.infs {
            self.log_gui(&format!("{:?}", inf), false, false);
        }
    }

    fn get_inf_list(&self) -> Result<String, String> {
        let mut r = String::new();
        if let Err(e) = 
            self.get_file_list(&Path::new(&self.infs_path), &mut r) 
        {
            return Err(e.to_string());
        } 
        if r.is_empty() {
            return Err(format!("No inf file name in {}", &self.infs_path));
        }
        Ok(r)
    }

    fn get_file_list(
        &self, 
        dir: &Path,
        mut list: &mut String
    ) -> Result<(), Box<dyn Error>> {
        if dir.is_dir() {
            match read_dir(dir) {
                Ok(result) => {
                    for entry in result {
                        let entry = entry.unwrap();
                        let path = entry.path();
                        if path.is_dir() {
                            self.get_file_list(&path, &mut list)?;
                        } 
                        else if let Some(ext) = path.extension() {
                            if ext.eq_ignore_ascii_case("inf") {
                                list.push_str(
                                    &format!(
                                        "{}\r\n", 
                                        &path.file_name().unwrap().to_string_lossy()
                                    )
                                );
                            }
                        }
                    }
                },
                Err(e) => {
                    return Err(Box::new(e));
                }
            }
        }

        Ok(())
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

    fn on_uninstall<'a, I>(&self, iter: I) 
        where I: Iterator<Item = &'a str>
    {
        let mut base_swcs: Vec<InfMetadata> = Vec::new();
        let mut exts: Vec<InfMetadata> = Vec::new();
        for to_uninstall in iter {
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

    fn confirm_uninstall(&mut self) {
        let main = self.window.clone().unwrap();
        let question = str_to_hstring("要移除這些inf嗎?");
        match pop_yesno(main.0, &question) {
            IDYES => {
                self.opts.force = true;
            },
            IDNO => {
                self.opts.force = false;
            },
            _ => {}
        }
    }

    fn proceed_uninstall(
        &self,
        base_swcs: &[InfMetadata], 
        exts: &[InfMetadata],
    ) {
        if base_swcs.is_empty() && exts.is_empty() {
            self.log("No inf files are removed.", self.opts.save_log, true, true);
            return;
        }

        let to_proceed: Vec<String> = Self::list_publish_names(&base_swcs);
        let total = to_proceed.len() + exts.len();
        let mut progress: (usize, usize, String);
        let mut curr = 0usize;
        let mut msg = String::new();
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
            curr += 1;
            let m = format!(
                "{}. {}={} of {} parent={}\r\n",
                curr, inf, org.original_name, org.class_name, pa.clone()
            ); 
            msg.push_str(&m);
            if self.opts.force {
                self.log(&m, self.opts.save_log, true, true);
                // let c = format!("pnputil /delete-driver {} /uninstall", &inf);
                // let res = cmd(&c);
                // self.log(res.as_ref().unwrap(), self.opts.save_log, false, true);
                //debug
                thread::sleep(Duration::from_millis(500));
                progress = (curr, total, format!("{}/{}", curr, total));
                self.post_message(Window::APP_UPDATE_PROGRESS, progress);
            }
        }

        for inf in exts.iter() {
            curr += 1;
            let m = format!( 
                "{}. {}={} of {}\r\n", 
                curr, inf.published_name, inf.original_name, inf.class_name
            );
            msg.push_str(&m);
            if self.opts.force {
                self.log(&m, self.opts.save_log, true, true);
                // let c = format!(
                //     "pnputil /delete-driver {} /uninstall", 
                //     &inf.published_name
                // );
                // let res = cmd(&c);
                // self.log(res.as_ref().unwrap(), self.opts.save_log, false, true);
                //debug
                thread::sleep(Duration::from_millis(300));
                progress = (curr, total, format!("{}/{}", curr, total));
                self.post_message(Window::APP_UPDATE_PROGRESS, progress);
            }
        }
        // list the results
        match self.opts.force {
            true => {
                self.log(
                    "The following inf files are removed.", 
                    self.opts.save_log, 
                    true, 
                    true
                );
                if self.window.is_some() {
                    // find DSP device on RPL MTL series CPU and remove 
                    // then re-scan for Intel SST OED
                    self.remove_dsp();
                    let info = "刪除完成\n請在Device Manager中確認driver已刪除";
                    self.post_message(Window::APP_POPUP_INFO, info.to_owned());
                }
            },
            false => {
                self.log(
                    "List the inf files:", 
                    self.opts.save_log, 
                    true, 
                    true
                );
            }
        }
        self.log(&msg, self.opts.save_log, false, true);
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

    fn post_message<T>(&self, msg: u32, data: T) {
        unsafe {
            let mut data = Box::new(data);
            let cds = COPYDATASTRUCT {
                dwData: 1, // Custom identifier
                cbData: mem::size_of::<T>() as u32,
                lpData: data.as_mut() as *mut _ as *mut core::ffi::c_void,
            };

            // Send with WM_USER
            if let Some(hwnd) = self.window.clone() {
                let hwnd = hwnd.0;
                // let _ = PostMessageW(hwnd, msg, WPARAM(0), LPARAM(0));
                let _ = SendMessageW(
                    hwnd,
                    msg,
                    WPARAM(&cds as *const _ as usize),
                    LPARAM(0)
                );
            }
        }
    }

    fn log(
        &self,
        content: &str, 
        save_file: bool, 
        add_time: bool, 
        on_screen: bool
    ) {
        match self.opts.gui_mode {
            true => {
                self.log_gui(content, add_time, on_screen);
            },
            false => {
                self.log_console(content, save_file, add_time, on_screen);
            }
        }
    }

    fn log_console(
        &self,
        content: &str, 
        save_file: bool, 
        add_time: bool, 
        on_screen: bool
    ) {
        let mut r: String = String::from(content);
        if add_time {
            let time_stamp = Local::now().format("%Y-%m%d_%H:%M:%S").to_string();
            r.insert_str(0, &format!("{}: ", &time_stamp));
        }
        if on_screen {
            println!("{}", &r);
        }
        if save_file {
            Logger::log(&r, &self.app_log).expect("Log to file failed.");
        }
    }

    fn log_gui(
        &self,
        content: &str, 
        add_time: bool, 
        on_screen: bool
    ) {
        let mut r = String::from(content);
        if add_time {
            let time_stamp = Local::now().format("%Y-%m%d_%H:%M:%S").to_string();
            r.insert_str(0, &format!("{}: ", &time_stamp));
        }
        if on_screen {
            self.post_message(Window::APP_UPDATE_RESULT, r.clone());
        }
        Logger::log(&r, &self.app_log).expect("Log to file failed.");
    }
}