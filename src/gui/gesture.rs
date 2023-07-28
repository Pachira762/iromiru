use super::macros::*;
use windows::Win32::Foundation::*;

pub struct Gesture {
    prev: Option<(i32, i32)>,
    delta: Option<(i32, i32)>,
}

impl Gesture {
    pub fn new() -> Self {
        Self {
            prev: None,
            delta: None,
        }
    }

    pub fn on_move(&mut self, wp: WPARAM, lp: LPARAM) -> Option<(i32, i32)> {
        if wp.0 == 1 {
            // left button

            let x = get_x_lp(lp);
            let y = get_y_lp(lp);

            if let Some((px, py)) = self.prev {
                self.delta = Some((x - px, y - py));
            } else {
                self.delta = None;
            }

            self.prev = Some((x, y));
        } else {
            self.prev = None;
            self.delta = None;
        }

        self.delta
    }
}
