use std::collections::HashMap;
use std::mem::zeroed;
use windows::core::*;
use windows::Foundation::*;
use windows::Win32::{
    Foundation::*,
    UI::{WindowsAndMessaging::*, Controls::*},
    System::LibraryLoader::*,
    Graphics::Gdi::*,
};
use crate::{
    app::App,
    win_str::*,
    dialog::*,
};

#[derive(Default)]
pub(crate) struct StrResource {
    pub(crate) path: HSTRING,
    pub(crate) remove: HSTRING,
}

impl StrResource {
    pub(crate) fn new() -> Self {
        Self {
            path: HSTRING::from("路徑"),
            remove: HSTRING::from("刪除"),
        }
    }
}

#[derive(Default)]
pub struct Window {
    app_wnd: HWND,
    controls: HashMap<usize, Rect>,
    app: App,
    local: StrResource,
    width: u32,
    height: u32,
}

impl Window {
    const ID_BTN_PATH: usize = 1;
    const ID_BTN_REMOVE: usize = 2;
    const ID_TEXTBOX_RESULT: usize = 3;
    const ID_TEXTBOX_PATH: usize = 4;
    const BTN_WIDTH: f32 = 80.0;
    const ONELINE_HEIGHT: f32 = 30.0;
    const PADDING: f32 = 5.0;

    pub fn new(title: &str, width: u32, height: u32, app: App) -> Result<Box<Self>> {
        unsafe {
            let instance = GetModuleHandleW(None)?;

            let window_class = w!("window");
            let wc = WNDCLASSW {
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                hInstance: instance.into(),
                lpszClassName: window_class,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wndproc),
                ..Default::default()
            };

            let atom = RegisterClassW(&wc);
            debug_assert!(atom != 0);

            let window_style = WS_OVERLAPPEDWINDOW | WS_VISIBLE;
            let (w, h) = {
                let mut rect = RECT {
                    left: 0,
                    top: 0,
                    right: width as i32,
                    bottom: height as i32,
                };
                AdjustWindowRect(&mut rect, window_style, false)?;
                (rect.right - rect.left, rect.bottom - rect.top)
            };

            let mut result = Box::new(
                Self {
                    app_wnd: HWND(std::ptr::null_mut()),
                    controls: HashMap::new(),
                    app,
                    local: StrResource::new(),
                    width,
                    height,
                    ..Default::default()
                }
            );

            // create main window
            result.app_wnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                window_class,
                &HSTRING::from(title),
                window_style,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                w,
                h,
                None,
                None,
                instance,
                Some(result.as_mut() as *mut _ as _),
            )?;

            let mut message = MSG::default();

            while GetMessageW(&mut message, None, 0, 0).into() {
                if !<BOOL as Into<bool>>::into(
                    IsDialogMessageW(result.app_wnd, &mut message)
                ) {
                    // translates keystrokes (key down, key up) into characters
                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }

            Ok(result)
        }
    }

    extern "system" fn wndproc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            if message == WM_NCCREATE {
                let cs = lparam.0 as *const CREATESTRUCTW;
                let this = (*cs).lpCreateParams as *mut Self;
                (*this).app_wnd = window;

                SetWindowLongPtrW(window, GWLP_USERDATA, this as _);
            } else {
                let this = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut Self;

                if !this.is_null() {
                    return (*this).message_handler(message, wparam, lparam);
                }
            }

            DefWindowProcW(window, message, wparam, lparam)
        }
    }

    fn message_handler(
        &mut self, message: u32, wparam: WPARAM, lparam: LPARAM
    ) -> LRESULT {
        unsafe {
            match message {
                WM_CREATE => {
                    let _ = self.build_ui();
                    self.init();
                    LRESULT(0)
                },
                WM_DESTROY => {
                    PostQuitMessage(0);
                    LRESULT(0)
                },
                WM_PAINT => {
                    self.on_paint();
                    LRESULT(0)
                },
                WM_SIZE => {
                    self.update_rect(lparam);
                    LRESULT(0)
                },
                WM_COMMAND => {
                    match wparam.0 {
                        Self::ID_BTN_PATH => {
                            self.on_path_btn();
                        },
                        Self::ID_BTN_REMOVE => {
                            self.app.remove_btn_click();
                        },
                        _ => {
                            self.on_textbox(wparam); 
                        },
                    }
                    LRESULT(0)
                },
                _ => DefWindowProcW(self.app_wnd, message, wparam, lparam),
            }
        }
    }

    fn on_textbox(&mut self, wparam: WPARAM) {
        let notification = Self::hiword(wparam.0 as isize) as u32;
        let control_id = Self::loword(wparam.0 as isize) as usize;
        
        match notification {
            EN_CHANGE => {
                // Identify which textbox changed
                match control_id {
                    Self::ID_TEXTBOX_PATH => {
                        unsafe {
                            if let Ok(control_hwnd) = 
                                GetDlgItem(self.app_wnd, control_id as i32) 
                            {
                                let text_length = 
                                    GetWindowTextLengthW(control_hwnd) + 1;
                                let mut buffer = vec![0u16; text_length as usize];
                                GetWindowTextW(control_hwnd, &mut buffer);
                                self.app.set_path(
                                    String::from_utf16_lossy(&buffer)
                                        .trim_end_matches('\0')
                                );
                            }
                        }
                    }
                    _ => {}
                }
            },
            _ => {}
        }
    }

    fn on_paint(&self) {
        unsafe {
            // repaint whole window
            let mut ps: PAINTSTRUCT = zeroed();
            let hdc = BeginPaint(self.app_wnd, &mut ps);
            let mut rect: RECT = zeroed();
            GetClientRect(self.app_wnd, &mut rect).unwrap();
            FillRect(hdc, &rect, GetSysColorBrush(COLOR_WINDOW));
            EndPaint(self.app_wnd, &ps).unwrap();
        }
        // redraw controls
        self.update_position();
    }

    fn build_ui(&mut self) -> Result<()> {
        unsafe {
            let instance = GetModuleHandleW(None)?;
            // Create path textbox
            let path_tb_rect = Rect {
                X: Self::PADDING, 
                Y: Self::PADDING, 
                Width: self.width as f32 - Self::BTN_WIDTH * 2.0 - Self::PADDING * 4.0, 
                Height: Self::ONELINE_HEIGHT
            };
            self.controls.insert(Self::ID_TEXTBOX_PATH, path_tb_rect);
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("EDIT"),
                w!(""),
                WINDOW_STYLE(
                    WS_VISIBLE.0 | 
                    WS_CHILD.0 | 
                    WS_BORDER.0 |
                    ES_AUTOHSCROLL as u32
                ),
                path_tb_rect.X as i32,
                path_tb_rect.Y as i32,
                path_tb_rect.Width as i32,
                path_tb_rect.Height as i32,
                self.app_wnd,
                HMENU(Self::ID_TEXTBOX_PATH as _),
                instance,
                None,
            )?;

            // Create result textbox
            let result_tb_rect = Rect {
                X: Self::PADDING, 
                Y: path_tb_rect.Y + Self::ONELINE_HEIGHT + Self::PADDING, 
                Width: self.width as f32 - Self::PADDING * 2.0, 
                Height: self.height as f32 - 
                    Self::ONELINE_HEIGHT - 
                    path_tb_rect.Y - 
                    Self::PADDING * 2.0
            };
            self.controls.insert(Self::ID_TEXTBOX_RESULT, result_tb_rect);
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("EDIT"),
                w!(""),
                WINDOW_STYLE(
                    WS_VISIBLE.0 | 
                    WS_CHILD.0 | 
                    WS_BORDER.0 | 
                    WS_VSCROLL.0 | 
                    ES_MULTILINE as u32 | 
                    ES_AUTOVSCROLL as u32 | 
                    ES_READONLY as u32
                ),
                result_tb_rect.X as i32,
                result_tb_rect.Y as i32,
                result_tb_rect.Width as i32,
                result_tb_rect.Height as i32,
                self.app_wnd,
                HMENU(Self::ID_TEXTBOX_RESULT as _),
                instance,
                None,
            )?;

            // Create path button
            let path_btn_rect = Rect {
                X: path_tb_rect.X + path_tb_rect.Width + Self::PADDING, 
                Y: path_tb_rect.Y, 
                Width: Self::BTN_WIDTH, 
                Height: Self::ONELINE_HEIGHT
            };
            self.controls.insert(Self::ID_BTN_PATH, path_btn_rect);
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                hstr_to_pcwstr(&self.local.path),
                WS_VISIBLE | WS_CHILD,
                path_btn_rect.X as i32,
                path_btn_rect.Y as i32,
                path_btn_rect.Width as i32,
                path_btn_rect.Height as i32,
                self.app_wnd,
                HMENU(Self::ID_BTN_PATH as _),
                instance,
                None,
            )?;

            // Create remove button
            let remove_btn_rect = Rect {
                X: path_btn_rect.X + path_btn_rect.Width + Self::PADDING, 
                Y: path_tb_rect.Y, 
                Width: Self::BTN_WIDTH, 
                Height: Self::ONELINE_HEIGHT
            };
            self.controls.insert(Self::ID_BTN_REMOVE, remove_btn_rect);
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                hstr_to_pcwstr(&self.local.remove),
                WS_VISIBLE | WS_CHILD,
                remove_btn_rect.X as i32,
                remove_btn_rect.Y as i32,
                remove_btn_rect.Width as i32,
                remove_btn_rect.Height as i32,
                self.app_wnd,
                HMENU(Self::ID_BTN_REMOVE as _),
                instance,
                None,
            )?;
        }
        Ok(())
    }

    fn update_rect(&mut self, lparam: LPARAM) {
        if self.controls.is_empty() {
            return;
        }
        let (width, height) = (Self::loword(lparam.0), Self::hiword(lparam.0));
        // update path textbox
        let path_tb_rect = self.controls.get_mut(&Self::ID_TEXTBOX_PATH).unwrap();
        path_tb_rect.Width = width as f32 - Self::BTN_WIDTH * 2.0 - Self::PADDING * 4.0;
        // update result textbox
        let path_tb_rect = self.controls.get(&Self::ID_TEXTBOX_PATH).cloned().unwrap();
        let rect = self.controls.get_mut(&Self::ID_TEXTBOX_RESULT).unwrap();
        rect.Width = width as f32 - 10.0; 
        rect.Height = height as f32 - 15.0 - path_tb_rect.Height;
        // update path button
        let rect = self.controls.get_mut(&Self::ID_BTN_PATH).unwrap();
        rect.X = path_tb_rect.X + path_tb_rect.Width + Self::PADDING;
        // update remove button
        let rect = self.controls.get_mut(&Self::ID_BTN_REMOVE).unwrap();
        rect.X = path_tb_rect.X + path_tb_rect.Width + Self::BTN_WIDTH + Self::PADDING * 2.0;
    }

    fn update_position(&self) {
        unsafe {
            self.controls.iter().for_each(|(id, rect)| {
                if let Ok(hwnd) = GetDlgItem(self.app_wnd, *id as i32) {
                    let _ = SetWindowPos(
                        hwnd,
                        None,
                        rect.X as i32,
                        rect.Y as i32,
                        rect.Width as i32,
                        rect.Height as i32,
                        SWP_NOZORDER | SWP_NOOWNERZORDER,
                    );
                }
                // scroll path textbox to start position 
                if let Ok(hwnd) = GetDlgItem(self.app_wnd, Self::ID_TEXTBOX_PATH as i32) {
                    SendMessageW(hwnd, EM_SETSEL, WPARAM(0), LPARAM(0));
                }
            });
        }
    }

    fn on_path_btn(&mut self) {
        if let Ok(s) = select_folder() {
            if s.is_empty() {
                return;
            }
            self.app.set_path(&s);
            // display the selected path
            self.set_path_text(&HSTRING::from(&s));
        }
    }

    fn set_path_text(&self, path: &HSTRING) {
        unsafe {
            if let Ok(textbox) = 
                GetDlgItem(self.app_wnd, Self::ID_TEXTBOX_PATH as i32) 
            {
                let _ = SetWindowTextW(textbox, hstr_to_pcwstr(&path));
            }
        }
    }

    fn init(&mut self) {
        unsafe {
            self.set_path_text(&self.app.get_infs_path());
            if let Ok(textbox) = 
                GetDlgItem(self.app_wnd, Self::ID_TEXTBOX_RESULT as i32) 
            {
                self.app.set_result_tb(&textbox);
                self.app.init_gui();
            }
        }
    }

    fn loword(l: isize) -> isize {
        l & 0xffff
    }

    fn hiword(l: isize) -> isize {
        (l >> 16) & 0xffff
    }
}