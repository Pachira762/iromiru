use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub fn cursor_pos() -> (i32, i32) {
    unsafe {
        let mut p = POINT::default();
        GetCursorPos(&mut p);
        (p.x, p.y)
    }
}

pub fn module_handle() -> HMODULE {
    unsafe { GetModuleHandleA(None).expect("failed to get current module handle.") }
}
