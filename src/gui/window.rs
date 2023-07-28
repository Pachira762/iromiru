use super::dpi::Dpi;
use super::macros::*;
use std::ffi::c_void;
use std::ptr::NonNull;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

#[repr(transparent)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Window(pub HWND);

impl Window {
    pub fn create(
        classname: PCSTR,
        name: PCSTR,
        exstyle: WINDOW_EX_STYLE,
        style: WINDOW_STYLE,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        parent: HWND,
        hmenu: HMENU,
        param: Option<*const c_void>,
    ) -> Result<Self> {
        unsafe {
            let hwnd = CreateWindowExA(
                exstyle,
                classname,
                name,
                style,
                x,
                y,
                cx,
                cy,
                parent,
                hmenu,
                GetModuleHandleA(None)?,
                param,
            );

            if hwnd == HWND(0) {
                Err(Error::from_win32())
            } else {
                Ok(Self(hwnd))
            }
        }
    }

    pub fn create_with_class(
        wc: &WNDCLASSEXA,
        name: PCSTR,
        exstyle: WINDOW_EX_STYLE,
        style: WINDOW_STYLE,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        parent: HWND,
        hmenu: HMENU,
        param: Option<*const c_void>,
    ) -> Result<Self> {
        unsafe {
            if RegisterClassExA(wc) == 0 {
                Err(Error::from_win32())
            } else {
                Self::create(
                    wc.lpszClassName,
                    name,
                    exstyle,
                    style,
                    x,
                    y,
                    cx,
                    cy,
                    parent,
                    hmenu,
                    param,
                )
            }
        }
    }

    #[allow(unused)]
    pub fn hwnd(&self) -> HWND {
        self.0
    }

    #[allow(unused)]
    pub fn is_null(&self) -> bool {
        self.0 == HWND(0)
    }

    #[allow(unused)]
    pub fn owner(&self) -> Window {
        unsafe { Window(GetWindow(self.0, GW_OWNER)) }
    }

    #[allow(unused)]
    pub fn dpi(&self) -> Dpi {
        Dpi(unsafe { GetDpiForWindow(self.0) } as _)
    }

    #[allow(unused)]
    pub fn position(&self) -> (i32, i32) {
        let RECT { left, top, .. } = self.rect();
        (left, top)
    }

    #[allow(unused)]
    pub fn set_position(&self, x: i32, y: i32) {
        unsafe {
            SetWindowPos(
                self.0,
                None,
                x,
                y,
                0,
                0,
                SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOSIZE,
            );
        }
    }

    #[allow(unused)]
    pub fn size(&self) -> (i32, i32) {
        rect_size(&self.rect())
    }

    #[allow(unused)]
    pub fn set_size(&self, width: i32, height: i32) {
        unsafe {
            SetWindowPos(
                self.0,
                None,
                0,
                0,
                width,
                height,
                SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOMOVE,
            );
        }
    }

    #[allow(unused)]
    pub fn rect(&self) -> RECT {
        unsafe {
            let mut rect = Default::default();
            GetWindowRect(self.0, &mut rect);
            rect
        }
    }

    #[allow(unused)]
    pub fn set_rect(&self, rect: &RECT) {
        unsafe {
            SetWindowPos(
                self.0,
                None,
                rect.left,
                rect.top,
                rect_width(&rect),
                rect_height(&rect),
                SWP_NOACTIVATE | SWP_NOZORDER,
            );
        }
    }

    #[allow(unused)]
    pub fn client_position(&self) -> (i32, i32) {
        unsafe {
            let mut position = Default::default();
            ClientToScreen(self.0, &mut position);
            (position.x, position.y)
        }
    }

    #[allow(unused)]
    pub fn client_size(&self) -> (i32, i32) {
        unsafe {
            let mut rect = Default::default();
            GetClientRect(self.0, &mut rect);
            rect_size(&rect)
        }
    }

    #[allow(unused)]
    pub fn client_rect(&self) -> RECT {
        let (left, top) = self.client_position();
        let (width, height) = self.client_size();
        RECT {
            left,
            top,
            right: left + width,
            bottom: top + height,
        }
    }

    #[allow(unused)]
    pub fn is_visible(&self) -> bool {
        unsafe { IsWindowVisible(self.0) != BOOL(0) }
    }

    #[allow(unused)]
    pub fn set_visibility(&self, visible: bool) {
        unsafe {
            ShowWindow(self.0, if visible { SW_SHOWNORMAL } else { SW_HIDE });
        }
    }

    #[allow(unused)]
    pub fn is_enabled(&self) -> bool {
        unsafe { IsWindowEnabled(self.0).into() }
    }

    #[allow(unused)]
    pub fn set_enabled(&self, enabled: bool) {
        unsafe {
            EnableWindow(self.0, enabled);
        }
    }

    #[allow(unused)]
    pub fn text(&self) -> [u8; 256] {
        unsafe {
            let mut buf = [0; 256];
            GetWindowTextA(self.0, &mut buf);
            buf
        }
    }

    #[allow(unused)]
    pub fn set_text(&self, text: PCSTR) {
        unsafe {
            _ = SetWindowTextA(self.0, text);
        }
    }

    #[allow(unused)]
    pub fn scroll(&self, dx: i32, dy: i32, flags: SCROLL_WINDOW_FLAGS) -> i32 {
        unsafe { ScrollWindowEx(self.0, dx, dy, None, None, None, None, flags) }
    }

    #[allow(unused)]
    pub fn is_checked(&self) -> bool {
        self.send_message(BM_GETCHECK, WPARAM(0), LPARAM(0)).0 as u32 == BST_CHECKED.0
    }

    #[allow(unused)]
    pub fn set_check(&self, check: bool) {
        self.send_message(
            BM_SETCHECK,
            WPARAM(if check { BST_CHECKED } else { BST_UNCHECKED }.0 as _),
            LPARAM(0),
        );
    }

    #[allow(unused)]
    pub fn set_timer(&self, id: usize, elapse: u32) {
        unsafe {
            SetTimer(self.0, id, elapse, None);
        }
    }

    #[allow(unused)]
    pub fn set_transparency(&self, transparent: bool) {
        unsafe {
            let style = WINDOW_EX_STYLE(GetWindowLongPtrA(self.0, GWL_EXSTYLE) as _);

            if transparent && (style & WS_EX_TRANSPARENT).0 == 0 {
                SetWindowLongPtrA(self.0, GWL_EXSTYLE, (style | WS_EX_TRANSPARENT).0 as _);
            } else if !transparent && (style & WS_EX_TRANSPARENT).0 != 0 {
                SetWindowLongPtrA(self.0, GWL_EXSTYLE, (style & (!WS_EX_TRANSPARENT)).0 as _);
            }
        }
    }

    #[allow(unused)]
    pub fn enable_layered(&self) -> Result<()> {
        unsafe {
            if SetLayeredWindowAttributes(self.0, COLORREF(0), 255, LWA_ALPHA) == FALSE {
                Err(Error::from_win32())
            } else {
                Ok(())
            }
        }
    }

    #[allow(unused)]
    pub fn set_display_affinity(&self, affinity: WINDOW_DISPLAY_AFFINITY) -> Result<()> {
        unsafe {
            if SetWindowDisplayAffinity(self.0, affinity) == FALSE {
                Err(Error::from_win32())
            } else {
                Ok(())
            }
        }
    }

    pub fn apply_dark_mode(&self) {
        unsafe {
            _ = SetWindowTheme(self.0, w!("DarkMode_Explorer"), None);
        }
    }

    pub fn set_font(&self, font: HFONT) {
        self.send_message(WM_SETFONT, WPARAM(font.0 as _), LPARAM(true.into()));
    }

    #[allow(unused)]
    pub fn send_message(&self, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
        unsafe { SendMessageA(self.0, msg, wp, lp) }
    }

    #[allow(unused)]
    pub fn post_message(&self, msg: u32, wp: WPARAM, lp: LPARAM) -> Result<()> {
        unsafe {
            if PostMessageA(self.0, msg, wp, lp) == FALSE {
                Err(Error::from_win32())
            } else {
                Ok(())
            }
        }
    }

    #[allow(unused)]
    pub fn embed_create_parameter(&self, lp: LPARAM) {
        unsafe {
            let cs: &CREATESTRUCTA = std::mem::transmute(lp);
            SetWindowLongPtrA(self.0, GWLP_USERDATA, cs.lpCreateParams as _);
        }
    }

    #[allow(unused)]
    pub fn retrieve_object<T>(&self) -> Option<NonNull<T>> {
        unsafe { NonNull::new(GetWindowLongPtrA(self.0, GWLP_USERDATA) as *mut T) }
    }
}
