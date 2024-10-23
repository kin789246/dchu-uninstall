use windows::core::*;
use windows::Win32::{
    Foundation::*,
    UI::{
        WindowsAndMessaging::*,
        Input::KeyboardAndMouse::*,
    },
    System::LibraryLoader::*,
    Graphics::Gdi::*,
};
use std::mem::zeroed;
use win_str::*;
use crate::*;

#[derive(Default)]
pub struct StateData {
    text: HSTRING,
}

pub(crate) struct StrResource {
    pub(crate) open: HSTRING,
    pub(crate) save: HSTRING,
    pub(crate) exit: HSTRING,
    pub(crate) file: HSTRING,
    pub(crate) btn_txt: HSTRING,
    pub(crate) clicked: HSTRING,
}

impl StrResource {
    pub(crate) fn new() -> Self {
        Self {
            open: HSTRING::from("開啟"),
            save: HSTRING::from("儲存"),
            exit: HSTRING::from("離開"),
            file: HSTRING::from("檔案"),
            btn_txt: HSTRING::from("好"),
            clicked: HSTRING::from("你按了!")
        }
    }
}

pub struct Wndrs {
    handle: HWND,
    title: HSTRING, 
    state: StateData,
    local: StrResource,
}

impl Wndrs {
    const ID_BTN_LOAD: usize = 1;
    const ID_MENU_OPEN: usize = 2;
    const ID_MENU_SAVE: usize = 3;
    const ID_MENU_EXIT: usize = 4;
    const ID_BTN_2: usize = 5;

    pub fn new(t: &str) -> Result<Self> {
        Ok(
            Wndrs { 
                handle: Default::default(),
                title: HSTRING::from(t),
                state: StateData { text: HSTRING::from("測試RUST中文介面") },
                local: StrResource::new(),
            }
        )
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
                (*this).handle = window;

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
                WM_DESTROY => {
                    PostQuitMessage(0);
                    LRESULT(0)
                },
                WM_PAINT => {
                    let mut ps: PAINTSTRUCT = zeroed();
                    let hdc = BeginPaint(self.handle, &mut ps);
                    let mut rect: RECT = zeroed();
                    GetClientRect(self.handle, &mut rect).unwrap();
                    FillRect(hdc, &rect, GetSysColorBrush(COLOR_WINDOW));
                    DrawTextW(
                        hdc,
                        self.state.text.as_wide().to_vec().as_mut(),
                        &mut rect,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_WORD_ELLIPSIS,
                    );
                    EndPaint(self.handle, &ps).unwrap();
                    LRESULT(0)
                },
                WM_MENUCHAR => {
                    // The high-order word of wParam contains the character that was pressed
                    let _char_code = (wparam.0 >> 16) as u16;
                    
                    // The low-order word of wParam indicates the menu flag
                    let _menu_flag = (wparam.0 & 0xFFFF) as u16;
                    
                    // lparam contains a handle to the menu
                    let _hmenu = HMENU(lparam.0 as _);

                    // Here you would typically check if the character matches any of your menu accelerators
                    // For this example, we're just going to close the menu
                    
                    // Return MNC_CLOSE to close the menu
                    // The high-order word is 0 (no item matched)
                    // The low-order word is MNC_CLOSE
                    LRESULT((0 << 16) | MNC_SELECT as isize)
                },
                WM_COMMAND => {
                    match wparam.0 as usize {
                        Self::ID_MENU_OPEN => {
                            dialog::pop_info(self.handle, &self.local.open);
                        },
                        Self::ID_MENU_SAVE => {
                            dialog::pop_info(self.handle, &self.local.save);
                        },
                        Self::ID_BTN_LOAD => {
                            dialog::pop_info(self.handle, &self.local.clicked);
                        },
                        Self::ID_MENU_EXIT => {
                            PostQuitMessage(0);
                        },
                        Self::ID_BTN_2 => {
                            let _ = dialog::file_open();
                        }
                        _ => (),
                    }
                    LRESULT(0)
                },
                WM_SYSKEYDOWN => {
                    if wparam.0 as u16 == VK_MENU.0 { // VK_MENU is the virtual key code for the Alt key
                        // Show the menu
                        let hmenu = GetMenu(self.handle);
                        if !hmenu.is_invalid() {
                            // Activate the menu
                            SendMessageW(self.handle, WM_SYSCOMMAND, wparam, lparam);
                        }
                    }
                    LRESULT(0)
                },
                _ => DefWindowProcW(self.handle, message, wparam, lparam),
            }
        }
    }

    pub fn build(&mut self) -> Result<()> {
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

            let handle = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                window_class,
                PCWSTR::from_raw(self.title.as_ptr()),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                None,
                None,
                instance,
                Some(self as *mut _ as _),
            )?;
            
            control::creat_button(
                handle, 
                instance, 
                hstr_to_pcwstr(&self.local.btn_txt), 
                10, 
                10, 
                40, 
                30,
                HMENU(Self::ID_BTN_LOAD as _)
            )?;
            
            control::creat_button(
                handle, 
                instance, 
                hstr_to_pcwstr(&self.local.btn_txt), 
                100, 
                10, 
                40, 
                30,
                HMENU(Self::ID_BTN_2 as _)
            )?;

            // Attach the menu to the window
            let hmenu = self.create_menu()?;
            SetMenu(handle, hmenu)?;

            debug_assert!(!handle.is_invalid());
            debug_assert!(handle == self.handle);
            let mut message = MSG::default();

            while GetMessageW(&mut message, None, 0, 0).into() {
                if !<BOOL as Into<bool>>::into(
                    IsDialogMessageW(handle, &mut message)
                ) {
                    // translates keystrokes (key down, key up) into characters
                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
            Ok(())
        }
    }

    fn create_menu(&self) -> Result<HMENU> {
        unsafe {
            // Create the main menu
            let hmenu = CreateMenu()?;

            // Create a submenu for "File"
            let hmenu_file = CreatePopupMenu()?;
            AppendMenuW(
                hmenu_file, 
                MF_STRING, 
                Self::ID_MENU_OPEN, 
                hstr_to_pcwstr(&self.local.open)
            )?;
            AppendMenuW(
                hmenu_file, 
                MF_STRING, 
                Self::ID_MENU_SAVE, 
                hstr_to_pcwstr(&self.local.save)
            )?;
            AppendMenuW(hmenu_file, MF_SEPARATOR, 0, str_to_pcwstr(""))?;
            AppendMenuW(
                hmenu_file, 
                MF_STRING, 
                Self::ID_MENU_EXIT, 
                hstr_to_pcwstr(&self.local.exit)
            )?;

            // Attach the submenu to the main menu
            AppendMenuW(hmenu, 
                MF_POPUP, 
                hmenu_file.0 as usize, 
                hstr_to_pcwstr(&self.local.file)
            )?;

            // Return the menu handle
            Ok(hmenu)
        }
    }
}