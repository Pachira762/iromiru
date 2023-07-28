use std::mem::*;
use std::sync::RwLock;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub struct Theme {
    hbrush: HBRUSH,
    hfont: RwLock<HFONT>,
}

impl Drop for Theme {
    fn drop(&mut self) {
        unsafe {
            DeleteObject(self.hbrush);
            DeleteObject(self.font());
        }
    }
}

impl Theme {
    pub const WINDOW_COLOR: COLORREF = COLORREF(0x181818);
    pub const TEXT_COLOR: COLORREF = COLORREF(0xf0f0f0);

    pub fn new() -> Self {
        unsafe {
            let hbrush = CreateSolidBrush(Self::WINDOW_COLOR);
            let hfont = RwLock::new(HFONT(0));

            Self { hbrush, hfont }
        }
    }

    pub fn brush(&self) -> HBRUSH {
        self.hbrush
    }

    pub fn font(&self) -> HFONT {
        if let Ok(hfont) = self.hfont.read() {
            *hfont
        } else {
            HFONT(0)
        }
    }

    pub fn create_font_for_dpi(&self, dpi: u32) {
        unsafe {
            let hfont = self.font();
            DeleteObject(hfont);

            let mut lgfont = LOGFONTW::default();
            let _ = SystemParametersInfoForDpi(
                SPI_GETICONTITLELOGFONT.0 as _,
                size_of_val(&lgfont) as _,
                Some(&mut lgfont as *mut _ as _),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0).0 as _,
                dpi,
            );

            if let Ok(mut hfont) = self.hfont.write() {
                *hfont = CreateFontIndirectW(&lgfont);
            }
        }
    }
}
