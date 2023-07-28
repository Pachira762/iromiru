use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::CREATESTRUCTA};

#[allow(unused)]
pub fn make_long(lo: u16, hi: u16) -> usize {
    (lo as usize & 0xffff) | ((hi as usize & 0xffff) << 16)
}

#[allow(unused)]
pub fn loword(l: usize) -> u16 {
    (l & 0xffff) as _
}

#[allow(unused)]
pub fn hiword(l: usize) -> u16 {
    ((l >> 16) & 0xffff) as _
}

#[allow(unused)]
pub fn break_wp(lp: WPARAM) -> (u16, u16) {
    (loword_wp(lp), hiword_wp(lp))
}

#[allow(unused)]
pub fn make_wp(lo: u16, hi: u16) -> WPARAM {
    WPARAM(make_long(lo, hi))
}

#[allow(unused)]
pub fn loword_wp(wp: WPARAM) -> u16 {
    loword(wp.0 as _)
}

#[allow(unused)]
pub fn hiword_wp(wp: WPARAM) -> u16 {
    hiword(wp.0 as _)
}

#[allow(unused)]
pub fn make_lp(lo: u16, hi: u16) -> LPARAM {
    LPARAM(make_long(lo, hi) as _)
}

#[allow(unused)]
pub fn break_lp(lp: LPARAM) -> (u16, u16) {
    (lp_loword(lp), lp_hiword(lp))
}

#[allow(dead_code)]
pub(super) fn lp_loword(lp: LPARAM) -> u16 {
    loword(lp.0 as _)
}

#[allow(dead_code)]
pub(super) fn lp_hiword(lp: LPARAM) -> u16 {
    hiword(lp.0 as _)
}

#[allow(unused)]
pub fn get_x_lp(lp: LPARAM) -> i32 {
    (lp_loword(lp) as i16) as i32
}

#[allow(unused)]
pub fn get_y_lp(lp: LPARAM) -> i32 {
    (lp_hiword(lp) as i16) as i32
}

#[allow(unused)]
pub fn rect_width(rect: &RECT) -> i32 {
    rect.right - rect.left
}

#[allow(unused)]
pub fn rect_height(rect: &RECT) -> i32 {
    rect.bottom - rect.top
}

#[allow(unused)]
pub fn rect_size(rect: &RECT) -> (i32, i32) {
    (rect_width(rect), rect_height(rect))
}

#[allow(unused)]
pub fn wheel_delta(wp: WPARAM) -> i32 {
    ((wp.0 >> 16) & 0xffff) as i16 as i32
}

#[allow(unused)]
pub fn create_struct(lp: LPARAM) -> *const CREATESTRUCTA {
    unsafe { std::mem::transmute(lp.0) }
}

#[allow(unused)]
pub fn create_param<T>(lp: LPARAM) -> *mut T {
    unsafe { std::mem::transmute((*create_struct(lp)).lpCreateParams) }
}
