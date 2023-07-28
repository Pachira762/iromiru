use windows::Win32::UI::{
    HiDpi::GetSystemMetricsForDpi, WindowsAndMessaging::SYSTEM_METRICS_INDEX,
};

#[derive(Clone, Copy, Debug)]
pub struct Dpi(pub i32);

impl Dpi {
    pub fn absolute(&self, pix: i32) -> i32 {
        pix * self.0 / 96
    }

    pub fn metrics(&self, index: SYSTEM_METRICS_INDEX) -> i32 {
        unsafe { GetSystemMetricsForDpi(index, self.0 as u32) }
    }
}
