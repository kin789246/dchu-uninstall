use std::ffi::OsString;
use std::mem::zeroed;
use std::os::windows::ffi::OsStrExt;
use windows_sys::{
    core::*, 
    Win32::Foundation::*, 
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW, 
    Win32::UI::WindowsAndMessaging::*,
    Win32::UI::HiDpi::{SetProcessDpiAwareness, PROCESS_PER_MONITOR_DPI_AWARE}
};

const BTN_LOAD: i32 = 1;

#[derive(Default)]
pub struct WindowData {
    text: String,
}

fn wide_string(s: &str) -> Vec<u16> {
    OsString::from(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub fn pop_yesno(msg: &str) -> MESSAGEBOX_RESULT {
    unsafe {
        MessageBoxW(
            0 as HWND,
            wide_string(msg).as_ptr(),
            wide_string("Question").as_ptr(),
            MB_YESNO | MB_ICONQUESTION,
        )
    }
}

pub fn pop_info(msg: &str) -> MESSAGEBOX_RESULT {
    unsafe {
        MessageBoxW(
            0 as HWND,
            wide_string(msg).as_ptr(),
            wide_string("Information").as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        )
    }
}
extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            },
            WM_PAINT => {
                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowData;
                if ptr.is_null() {
                    return DefWindowProcW(hwnd, msg, wparam, lparam);
                }
                let data = &*ptr;
                let mut ps: PAINTSTRUCT = zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                // let mut rect = ps.rcPaint;
                let mut rect: RECT = zeroed();
                GetClientRect(hwnd, &mut rect);
                FillRect(hdc, &rect, GetStockObject(WHITE_BRUSH as i32) as HBRUSH);
                DrawTextW(
                    hdc,
                    wide_string(&data.text).as_ptr(),
                    -1,
                    &mut rect,
                    windows_sys::Win32::Graphics::Gdi::DT_CENTER
                        | windows_sys::Win32::Graphics::Gdi::DT_VCENTER
                        | windows_sys::Win32::Graphics::Gdi::DT_SINGLELINE
                        | windows_sys::Win32::Graphics::Gdi::DT_WORD_ELLIPSIS,
                );
                EndPaint(hwnd, &ps);
                0
            },
            WM_COMMAND => {
                let control_id = wparam as u16 as i32;
                if control_id == BTN_LOAD {
                    MessageBoxW(
                        hwnd, 
                        wide_string("Button clicked!").as_ptr(), 
                        wide_string("Info").as_ptr(), 
                        MB_OK
                    );
                }
                0
            },
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

pub fn create_window(title: &str, text: &str) {
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        debug_assert!(!instance.is_null());

        SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE);
        let window_class = w!("window");

        let wc = WNDCLASSW {
            hCursor: LoadCursorW(core::ptr::null_mut(), IDC_ARROW),
            hInstance: instance,
            lpszClassName: window_class,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: core::ptr::null_mut(),
            hbrBackground: core::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
        };

        let atom = RegisterClassW(&wc);
        debug_assert!(atom != 0);

        let hwnd = CreateWindowExW(
            0,
            window_class,
            wide_string(title).as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            instance,
            std::ptr::null(),
        );

        // create a button
        CreateWindowExW( 
            0,
            wide_string("BUTTON").as_ptr(),  // Predefined class; Unicode assumed 
            wide_string("OK").as_ptr(),      // Button text 
            WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_DEFPUSHBUTTON as u32,  // Styles 
            10,         // x position 
            10,         // y position 
            100,        // Button width
            50,        // Button height
            hwnd,       // Parent window
            BTN_LOAD as HMENU, // BUTTON_ID as menu.
            instance, 
            std::ptr::null() // Pointer not needed.
        );      

        // Sets the user data associated with the window.
        let data = Box::new(WindowData {
            text: text.to_string(),
        });
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(data) as isize);

        let mut message = std::mem::zeroed();

        while GetMessageW(&mut message, core::ptr::null_mut(), 0, 0) != 0 {
            // translates keystrokes (key down, key up) into characters
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}