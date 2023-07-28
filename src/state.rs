use glam::Quat;
use windows::Win32::Foundation::RECT;

pub use crate::gui::*;
use std::sync::{Arc, RwLock};

#[derive(Clone, Copy, Eq, PartialEq, Default, Hash, Debug)]
pub enum ColorSpace {
    #[default]
    Rgb,
    Hsv,
    Hsl,
    Yuv,
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct ChannelMask(pub [bool; 3]);

impl ChannelMask {
    pub fn new(ch1: bool, ch2: bool, ch3: bool) -> Self {
        Self([ch1, ch2, ch3])
    }

    pub fn at(&self, i: usize) -> bool {
        self.0.get(i).cloned().unwrap_or_default()
    }
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ViewMode {
    #[default]
    Original,
    Rgb(ChannelMask),
    Hue,
    Saturation,
    Brightness,
}

impl ViewMode {
    pub fn is_enable(&self) -> bool {
        match *self {
            Self::Original => false,
            _ => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum HistogramMode {
    #[default]
    Disable,
    Rgb,
    Hue,
    Saturation,
    Brightness,
}

impl HistogramMode {
    pub fn is_enable(&self) -> bool {
        match *self {
            Self::Disable => false,
            _ => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ColorCloudMode {
    #[default]
    Disable,
    Enable(ColorSpace),
}

impl ColorCloudMode {
    pub fn is_enable(&self) -> bool {
        match *self {
            Self::Disable => false,
            _ => true,
        }
    }

    pub fn color_space(&self) -> Option<ColorSpace> {
        match *self {
            ColorCloudMode::Disable => None,
            ColorCloudMode::Enable(color_space) => Some(color_space),
        }
    }
}

#[derive(Clone, Default)]
pub struct State {
    pub active: bool,
    pub rect: RECT,
    pub view_mode: ViewMode,
    pub histogram_mode: HistogramMode,
    pub color_cloud_mode: ColorCloudMode,
    pub rotation: Quat,
}

impl State {
    pub fn move_camera(&mut self, dx: f32, dy: f32) {
        self.rotation *= Quat::from_rotation_x((180.0 * dy).to_radians());
        self.rotation *= Quat::from_rotation_y((180.0 * dx).to_radians());
    }
}

#[derive(Clone)]
pub struct RefState(pub Arc<RwLock<State>>);

macro_rules! impl_accessor {
    ($name:ident: $type:ty, $getter:ident, $setter:ident) => {
        #[allow(unused)]
        pub fn $getter(&self) -> $type {
            if let Ok(state) = self.0.read() {
                state.$name
            } else {
                Default::default()
            }
        }

        #[allow(unused)]
        pub fn $setter(&self, $name: $type) {
            if let Ok(mut state) = self.0.write() {
                state.$name = $name;
            }
        }
    };
}

impl RefState {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(State {
            active: true,
            rotation: Quat::IDENTITY,
            ..Default::default()
        })))
    }

    impl_accessor!(active: bool, is_active, set_active);

    impl_accessor!(view_mode: ViewMode, view_mode, set_view_mode);

    impl_accessor!(
        histogram_mode: HistogramMode,
        histogram_mode,
        set_histogram_mode
    );

    impl_accessor!(
        color_cloud_mode: ColorCloudMode,
        color_cloud_mode,
        set_color_cloud_mode
    );

    impl_accessor!(rotation: Quat, rotation, set_rotation);

    pub fn move_camera(&mut self, dx: f32, dy: f32) {
        if let Ok(mut state) = self.0.write() {
            state.move_camera(dx, dy);
        }
    }

    pub fn read(&self) -> State {
        match self.0.read() {
            Ok(state) => state.clone(),
            Err(_) => State::default(),
        }
    }
}
