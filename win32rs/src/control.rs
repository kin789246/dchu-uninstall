use windows::core::*;
use windows::Win32::{
    Foundation::*,
    UI::WindowsAndMessaging::*,
};

pub fn creat_button(
    handle: HWND, 
    instance: HMODULE,
    btn_txt: PCWSTR,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    btn_id: HMENU,
) -> Result<()> {
    unsafe {
        // create a button
        CreateWindowExW( 
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),  // Predefined class; Unicode assumed 
            btn_txt,      // Button text 
            WINDOW_STYLE(
                WS_TABSTOP.0 | 
                WS_VISIBLE.0 | 
                WS_CHILD.0 | 
                BS_DEFPUSHBUTTON as u32
            ),  // Styles 
            x,         // x position 
            y,         // y position 
            width,        // Button width
            height,        // Button height
            handle,       // Parent window
            btn_id, // BUTTON_ID as menu.
            instance, 
            None // Pointer not needed.
        )?;      
        Ok(())
    }
}