use super::macros::*;
use super::Window;
use std::mem::size_of;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub(super) struct Scrollbar {
    window: Window,
}

impl Scrollbar {
    pub fn new() -> Self {
        Self {
            window: Window(HWND(0)),
        }
    }

    pub fn bind(&mut self, window: Window, range: i32, page: i32) {
        self.window = window;
        self.update(Some(range), Some(page), None);
    }

    pub fn handle_scroll(&mut self, wp: WPARAM) -> LRESULT {
        let command = SCROLLBAR_COMMAND(loword_wp(wp) as _);
        let info = self.info();
        let mut pos = info.nPos;

        match command {
            SB_LINEUP => {
                pos -= 13;
            }
            SB_LINEDOWN => {
                pos += 13;
            }
            SB_PAGEUP => {
                pos -= info.nPage as i32;
            }
            SB_PAGEDOWN => {
                pos += info.nPage as i32;
            }
            SB_TOP => {
                pos = info.nMin;
            }
            SB_BOTTOM => {
                pos = info.nMax;
            }
            SB_THUMBTRACK => {
                pos = info.nTrackPos;
            }
            _ => {}
        }

        self.update(None, None, Some(pos));
        LRESULT(0)
    }

    pub fn handle_mouse_wheel(&mut self, wp: WPARAM) -> LRESULT {
        let info = self.info();
        let delta = -wheel_delta(wp) * 13 / 120;
        self.update(None, None, Some(info.nPos + delta));

        LRESULT::default()
    }

    pub fn set_range(&mut self, range: i32) {
        self.update(Some(range), None, None);
    }

    pub fn set_page(&mut self, page: i32) {
        self.update(None, Some(page), None);
    }

    fn update(&mut self, range: Option<i32>, page: Option<i32>, pos: Option<i32>) {
        let prev = self.info();
        self.set_info(range, page, pos);

        let cur = self.info();
        let dp = cur.nPos - prev.nPos;

        self.window
            .scroll(0, -dp, SW_ERASE | SW_INVALIDATE | SW_SCROLLCHILDREN);
    }

    fn info(&self) -> SCROLLINFO {
        unsafe {
            let mut info = SCROLLINFO {
                cbSize: size_of::<SCROLLINFO>() as _,
                fMask: SIF_ALL,
                ..Default::default()
            };
            GetScrollInfo(self.window.hwnd(), SB_VERT, &mut info);

            info
        }
    }

    fn set_info(&self, range: Option<i32>, page: Option<i32>, pos: Option<i32>) {
        unsafe {
            SetScrollInfo(
                self.window.hwnd(),
                SB_VERT,
                &SCROLLINFO {
                    cbSize: size_of::<SCROLLINFO>() as _,
                    fMask: range.as_ref().map_or(SCROLLINFO_MASK(0), |_| SIF_RANGE)
                        | page.as_ref().map_or(SCROLLINFO_MASK(0), |_| SIF_PAGE)
                        | pos.as_ref().map_or(SCROLLINFO_MASK(0), |_| SIF_POS),
                    nMin: 0,
                    nMax: range.unwrap_or(0),
                    nPage: page.unwrap_or(0) as _,
                    nPos: pos.unwrap_or(0),
                    nTrackPos: 0,
                },
                TRUE,
            )
        };
    }
}
