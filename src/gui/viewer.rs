use super::gesture::Gesture;
use super::panel::Panel;
use super::theme::*;
use super::*;
use crate::state::*;
use std::mem::*;
use std::rc::Rc;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub struct Viewer {
    pub window: Window,
    gesture: Gesture,
    panel: Box<Panel>,
    state: RefState,
    theme: Rc<Theme>,
}

impl Viewer {
    pub fn new(state: RefState) -> Box<Self> {
        let theme = Rc::new(Theme::new());

        Box::new(Self {
            window: Window(HWND(0)),
            gesture: Gesture::new(),
            panel: Panel::new(Rc::clone(&theme), RefState::clone(&state)),
            state,
            theme,
        })
    }

    pub fn create(self: &Box<Self>) -> Result<()> {
        match Window::create_with_class(
            unsafe {
                &WNDCLASSEXA {
                    cbSize: size_of::<WNDCLASSEXA>() as _,
                    style: CS_HREDRAW | CS_VREDRAW,
                    lpfnWndProc: Some(wndproc),
                    hInstance: module_handle(),
                    hIcon: LoadIconW(module_handle(), PCWSTR(1 as _))?,
                    hCursor: LoadCursorW(None, IDC_ARROW)?,
                    hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as _),
                    lpszClassName: s!("IroMiru_Viewer"),
                    ..Default::default()
                }
            },
            s!("IroMiru"),
            WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_NOREDIRECTIONBITMAP,
            WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            200,
            HWND(0),
            HMENU(0),
            Some(self.as_ref() as *const _ as _),
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn show(&self) {
        self.window.set_visibility(true);
    }

    fn init(&mut self, hwnd: HWND) -> Result<()> {
        self.window = Window(hwnd);
        self.window.enable_layered()?;
        self.window.set_timer(0x01, 100);
        self.window.set_display_affinity(WDA_EXCLUDEFROMCAPTURE)?;

        self.theme.create_font_for_dpi(self.window.dpi().0 as u32);
        self.panel.create(self.window)?;

        Ok(())
    }

    fn update_transparency_and_panel_visibility(&mut self) {
        match self.state.color_cloud_mode() {
            ColorCloudMode::Disable => {
                self.window
                    .set_transparency(on_nc_hit_test(self.window.hwnd()).0 as u32 == HTCLIENT);
            }
            _ => {
                self.window.set_transparency(false);
            }
        }

        self.panel.update_visibility();
    }

    fn on_create(&mut self, hwnd: HWND) -> LRESULT {
        match self.init(hwnd) {
            Ok(_) => LRESULT(0),
            Err(_) => LRESULT(-1),
        }
    }

    fn on_move(&mut self, lp: LPARAM) -> LRESULT {
        let (x, y) = break_lp(lp);
        self.panel.update_position(x as _, y as _);
        LRESULT(0)
    }

    fn on_size(&mut self) -> LRESULT {
        self.panel.update_size();
        LRESULT(0)
    }

    fn on_timer(&mut self) -> LRESULT {
        self.update_transparency_and_panel_visibility();
        LRESULT(0)
    }

    fn on_mouse_move(&mut self, wp: WPARAM, lp: LPARAM) -> LRESULT {
        if let Some((dx, dy)) = self.gesture.on_move(wp, lp) {
            let width = self.window.client_size().0.min(480) as f32;
            self.state.move_camera(dx as f32 / width, dy as f32 / width);
        }

        LRESULT(0)
    }

    fn handle_message(&mut self, hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> Option<LRESULT> {
        match msg {
            WM_CREATE => Some(self.on_create(hwnd)),
            WM_MOVE => Some(self.on_move(lp)),
            WM_SIZE => Some(self.on_size()),
            WM_TIMER => Some(self.on_timer()),
            WM_MOUSEMOVE => Some(self.on_mouse_move(wp, lp)),
            _ => None,
        }
    }
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    unsafe {
        if msg == WM_CREATE {
            Window(hwnd).embed_create_parameter(lp);
        }

        Window(hwnd)
            .retrieve_object::<Viewer>()
            .and_then(|mut this| this.as_mut().handle_message(hwnd, msg, wp, lp))
            .unwrap_or_else(|| match msg {
                WM_CLOSE => on_close(hwnd),
                WM_DESTROY => on_destroy(),
                WM_KEYDOWN => on_key_down(hwnd, wp),
                WM_NCCALCSIZE => on_nc_calc_size(hwnd, wp, lp),
                WM_NCHITTEST => on_nc_hit_test(hwnd),
                _ => DefWindowProcA(hwnd, msg, wp, lp),
            })
    }
}

fn on_close(hwnd: HWND) -> LRESULT {
    unsafe {
        DestroyWindow(hwnd);
    }
    LRESULT(0)
}

fn on_destroy() -> LRESULT {
    unsafe {
        PostQuitMessage(0);
    }
    LRESULT(0)
}

fn on_key_down(hwnd: HWND, wp: WPARAM) -> LRESULT {
    if wp.0 == VK_ESCAPE.0 as _ {
        unsafe {
            DestroyWindow(hwnd);
        }
    }
    LRESULT(0)
}

fn on_nc_calc_size(_hwnd: HWND, _wp: WPARAM, lp: LPARAM) -> LRESULT {
    let rect: &mut RECT = unsafe { std::mem::transmute(lp) };
    // rect.left += 1;
    //rect.top += 1;
    // rect.right -= 1;
    rect.bottom -= 1;
    return LRESULT(0);
}

fn on_nc_hit_test(hwnd: HWND) -> LRESULT {
    enum Region {
        Outside,
        Caption,
        FrameA,
        Client,
        FrameB,
    }
    use Region::*;

    impl Region {
        fn detect(pos: i32, beg: i32, end: i32, frame: i32, caption: i32) -> Self {
            if pos < beg || pos >= end {
                Outside
            } else if pos < beg + frame {
                FrameA
            } else if pos >= end - frame {
                FrameB
            } else if pos < beg + caption || pos >= end - caption {
                Caption
            } else {
                Client
            }
        }
    }

    let (x, y) = cursor_pos();
    let rect = Window(hwnd).rect();
    let frame = unsafe {
        let mut frame = Default::default();
        AdjustWindowRectExForDpi(
            &mut frame,
            WS_OVERLAPPEDWINDOW,
            FALSE,
            WINDOW_EX_STYLE(0),
            Window(hwnd).dpi().0 as _,
        );
        frame
    };

    let rx = Region::detect(x, rect.left, rect.right, -frame.left, -frame.top - 1);
    let ry = Region::detect(y, rect.top, rect.bottom, -frame.left, -frame.top - 1);

    match (rx, ry) {
        (Outside, _) | (_, Outside) => LRESULT(HTNOWHERE as _),
        (Caption, _) | (_, Caption) => LRESULT(HTCAPTION as _),
        (FrameA, FrameA) => LRESULT(HTTOPLEFT as _),
        (Client, FrameA) => LRESULT(HTTOP as _),
        (FrameB, FrameA) => LRESULT(HTTOPRIGHT as _),
        (FrameA, Client) => LRESULT(HTLEFT as _),
        (Client, Client) => LRESULT(HTCLIENT as _),
        (FrameB, Client) => LRESULT(HTRIGHT as _),
        (FrameA, FrameB) => LRESULT(HTBOTTOMLEFT as _),
        (Client, FrameB) => LRESULT(HTBOTTOM as _),
        (FrameB, FrameB) => LRESULT(HTBOTTOMRIGHT as _),
    }
}
