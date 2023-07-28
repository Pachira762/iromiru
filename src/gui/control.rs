use super::macros::*;
use super::widget::*;
use crate::state::*;
use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ControlId(pub u32);

pub const VIEW_ORIGINAL: ControlId = ControlId(1);
pub const VIEW_RGB: ControlId = ControlId(VIEW_ORIGINAL.0 + 1);
pub const VIEW_RGB_R: ControlId = ControlId(VIEW_RGB.0 + 1);
pub const VIEW_RGB_G: ControlId = ControlId(VIEW_RGB.0 + 2);
pub const VIEW_RGB_B: ControlId = ControlId(VIEW_RGB.0 + 3);
pub const VIEW_HUE: ControlId = ControlId(VIEW_RGB_B.0 + 1);
pub const VIEW_SATURATION: ControlId = ControlId(VIEW_RGB_B.0 + 2);
pub const VIEW_BRIGHTNESS: ControlId = ControlId(VIEW_RGB_B.0 + 3);
pub const HISTOGRAM_DISABLE: ControlId = ControlId(VIEW_BRIGHTNESS.0 + 1);
pub const HISTOGRAM_RGB: ControlId = ControlId(HISTOGRAM_DISABLE.0 + 1);
pub const HISTOGRAM_HUE: ControlId = ControlId(HISTOGRAM_DISABLE.0 + 2);
pub const HISTOGRAM_SATURAION: ControlId = ControlId(HISTOGRAM_DISABLE.0 + 3);
pub const HISTOGRAM_BRIGHTNESS: ControlId = ControlId(HISTOGRAM_DISABLE.0 + 4);
pub const COLOR_CLOUD_DISABLE: ControlId = ControlId(HISTOGRAM_BRIGHTNESS.0 + 1);
pub const COLOR_CLOUD_RGB: ControlId = ControlId(COLOR_CLOUD_DISABLE.0 + 1);
pub const COLOR_CLOUD_HSV: ControlId = ControlId(COLOR_CLOUD_DISABLE.0 + 2);
pub const COLOR_CLOUD_HSL: ControlId = ControlId(COLOR_CLOUD_DISABLE.0 + 3);
pub const COLOR_CLOUD_YUV: ControlId = ControlId(COLOR_CLOUD_DISABLE.0 + 4);

impl ControlId {
    pub fn from_wp(wp: WPARAM) -> Self {
        Self(loword_wp(wp) as _)
    }

    // pub fn from_view_mode(mode: ViewMode) -> Self {
    //     match mode {
    //         ViewMode::Original => VIEW_ORIGINAL,
    //         ViewMode::Rgb(_) => VIEW_RGB,
    //         ViewMode::Hue => VIEW_HUE,
    //         ViewMode::Saturation => VIEW_SATURATION,
    //         ViewMode::Brightness => VIEW_BRIGHTNESS,
    //     }
    // }

    // pub fn from_histogram_mode(mode: HistogramMode) -> Self {
    //     match mode {
    //         HistogramMode::Disable => HISTOGRAM_DISABLE,
    //         HistogramMode::Rgb => HISTOGRAM_RGB,
    //         HistogramMode::Hue => HISTOGRAM_HUE,
    //         HistogramMode::Saturation => HISTOGRAM_SATURAION,
    //         HistogramMode::Brightness => HISTOGRAM_BRIGHTNESS,
    //     }
    // }

    // pub fn from_color_cloud_mode(mode: ColorCloudMode) -> Self {
    //     match mode {
    //         ColorCloudMode::Disable => COLOR_CLOUD_DISABLE,
    //         ColorCloudMode::Enable(color_space) => match color_space {
    //             ColorSpace::Rgb => COLOR_CLOUD_RGB,
    //             ColorSpace::Hsv => COLOR_CLOUD_HSV,
    //             ColorSpace::Hsl => COLOR_CLOUD_HSL,
    //             ColorSpace::Yuv => COLOR_CLOUD_YUV,
    //         },
    //     }
    // }

    pub fn color_space(&self) -> ColorSpace {
        match *self {
            VIEW_RGB | COLOR_CLOUD_RGB => ColorSpace::Rgb,
            COLOR_CLOUD_HSV => ColorSpace::Hsv,
            COLOR_CLOUD_HSL => ColorSpace::Hsl,
            COLOR_CLOUD_YUV => ColorSpace::Yuv,
            _ => panic!("no associated color space."),
        }
    }
}

impl From<ControlId> for HMENU {
    fn from(value: ControlId) -> Self {
        HMENU(value.0 as _)
    }
}

impl Key for ControlId {}
