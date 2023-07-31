use super::control::*;
use super::scrollbar::Scrollbar;
use super::theme::Theme;
use super::widget::*;
use super::Window;
use crate::state::*;
use std::mem::*;
use std::rc::Rc;
use windows::core::*;
use windows::s;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub(super) struct Panel {
    theme: Rc<Theme>,
    window: Window,
    parent: Window,
    tree: Tree<ControlId>,
    scrollbar: Scrollbar,
    state: RefState,
}

impl Panel {
    pub fn new(theme: Rc<Theme>, state: RefState) -> Box<Self> {
        Box::new(Self {
            theme,
            window: Window(HWND(0)),
            parent: Window(HWND(0)),
            tree: Tree::new(),
            scrollbar: Scrollbar::new(),
            state: state.clone(),
        })
    }

    pub fn create(self: &mut Box<Self>, parent: Window) -> Result<()> {
        self.parent = parent;
        let rect = parent.rect();

        match Window::create_with_class(
            unsafe {
                &WNDCLASSEXA {
                    cbSize: size_of::<WNDCLASSEXA>() as _,
                    style: CS_HREDRAW | CS_VREDRAW,
                    lpfnWndProc: Some(wndproc),
                    hInstance: module_handle(),
                    hIcon: LoadIconW(None, IDI_APPLICATION)?,
                    hCursor: LoadCursorW(None, IDC_ARROW)?,
                    hbrBackground: self.theme.brush(),
                    lpszClassName: s!("IroMiru_Panel"),
                    ..Default::default()
                }
            },
            s!(""),
            WS_EX_LAYERED,
            WS_THICKFRAME | WS_VSCROLL | WS_CLIPCHILDREN,
            rect.left,
            rect.top,
            0,
            rect_height(&rect),
            parent.hwnd(),
            HMENU(0),
            Some(self.as_ref() as *const _ as _),
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn update_visibility(&self) {
        let (x, y) = cursor_pos();
        let rect = self.window.rect();
        let visible = rect.left <= x && x < rect.right && rect.top <= y && y < rect.bottom;
        self.window.set_visibility(visible);
    }

    pub fn update_position(&self, x: i32, y: i32) {
        self.window.set_position(x, y);
    }

    pub fn update_size(&mut self) {
        let mut rect = self.parent.rect();
        rect.right = rect.left + rect_width(&self.window.rect());

        self.window.set_rect(&rect);
    }

    fn init(&mut self, hwnd: HWND) -> Result<()> {
        self.window = Window(hwnd);
        let dpi = self.window.dpi();

        self.window.enable_layered()?;
        self.window.apply_dark_mode();
        self.window.set_display_affinity(WDA_EXCLUDEFROMCAPTURE)?;

        let (_, cy) = self.window.client_size();
        self.scrollbar.bind(self.window, 0, cy);

        self.build()?;
        let (widget_width, _) = self.tree.size();

        let (_, height) = self.window.size();
        self.window
            .set_size(widget_width + dpi.metrics(SM_CXVSCROLL), height);

        Ok(())
    }

    fn build(&mut self) -> Result<()> {
        let state = self.state.read();

        self.tree.view(
            self.window,
            Rc::clone(&self.theme),
            &Stack::new(
                (0, 0),
                Axis::Vertical,
                11,
                &[
                    &Stack::new(
                        (0, 0),
                        Axis::Vertical,
                        0,
                        &[
                            &Text::new((0, 0), s!("View")),
                            &Radio::new(
                                (0, 0),
                                state.view_mode == ViewMode::Original,
                                true,
                                s!("Origin"),
                                VIEW_ORIGINAL,
                                None,
                            ),
                            &Radio::new(
                                (0, 7),
                                match state.view_mode {
                                    ViewMode::Rgb(_) => true,
                                    _ => false,
                                },
                                false,
                                s!("RGB"),
                                VIEW_RGB,
                                Some(&Stack::new(
                                    (11, 0),
                                    Axis::Horizontal,
                                    0,
                                    &[
                                        &Check::new((0, 0), true, s!("R"), VIEW_RGB_R),
                                        &Check::new((0, 0), true, s!("G"), VIEW_RGB_G),
                                        &Check::new((0, 0), true, s!("B"), VIEW_RGB_B),
                                    ],
                                )),
                            ),
                            &Radio::new(
                                (0, 7),
                                state.view_mode == ViewMode::Hue,
                                false,
                                s!("Hue"),
                                VIEW_HUE,
                                None,
                            ),
                            &Radio::new(
                                (0, 7),
                                state.view_mode == ViewMode::Saturation,
                                false,
                                s!("Saturation"),
                                VIEW_SATURATION,
                                None,
                            ),
                            &Radio::new(
                                (0, 7),
                                state.view_mode == ViewMode::Brightness,
                                false,
                                s!("Brightness"),
                                VIEW_BRIGHTNESS,
                                None,
                            ),
                        ],
                    ),
                    &Stack::new(
                        (0, 0),
                        Axis::Vertical,
                        0,
                        &[
                            &Text::new((0, 0), s!("Histogram")),
                            &Radio::new(
                                (0, 5),
                                state.histogram_mode == HistogramMode::Disable,
                                true,
                                s!("Disable"),
                                HISTOGRAM_DISABLE,
                                None,
                            ),
                            &Radio::new(
                                (0, 7),
                                state.histogram_mode == HistogramMode::Rgb,
                                false,
                                s!("RGB"),
                                HISTOGRAM_RGB,
                                None,
                            ),
                            &Radio::new(
                                (0, 7),
                                state.histogram_mode == HistogramMode::Hue,
                                false,
                                s!("Hue"),
                                HISTOGRAM_HUE,
                                None,
                            ),
                            &Radio::new(
                                (0, 7),
                                state.histogram_mode == HistogramMode::Saturation,
                                false,
                                s!("Saturation"),
                                HISTOGRAM_SATURAION,
                                None,
                            ),
                            &Radio::new(
                                (0, 7),
                                state.histogram_mode == HistogramMode::Brightness,
                                false,
                                s!("Brightness"),
                                HISTOGRAM_BRIGHTNESS,
                                None,
                            ),
                        ],
                    ),
                    &Stack::new(
                        (0, 0),
                        Axis::Vertical,
                        0,
                        &[
                            &Text::new((0, 0), s!("IroSphere")),
                            &Radio::new(
                                (0, 5),
                                state.color_cloud_mode == ColorCloudMode::Disable,
                                true,
                                s!("Disable"),
                                COLOR_CLOUD_DISABLE,
                                None,
                            ),
                            &Radio::new(
                                (0, 7),
                                state.color_cloud_mode.color_space() == Some(ColorSpace::Rgb),
                                false,
                                s!("RGB"),
                                COLOR_CLOUD_RGB,
                                None,
                            ),
                            &Radio::new(
                                (0, 7),
                                state.color_cloud_mode.color_space() == Some(ColorSpace::Hsv),
                                false,
                                s!("HSV"),
                                COLOR_CLOUD_HSV,
                                None,
                            ),
                            &Radio::new(
                                (0, 7),
                                state.color_cloud_mode.color_space() == Some(ColorSpace::Hsl),
                                false,
                                s!("HSL"),
                                COLOR_CLOUD_HSL,
                                None,
                            ),
                            &Radio::new(
                                (0, 7),
                                state.color_cloud_mode.color_space() == Some(ColorSpace::Yuv),
                                false,
                                s!("YUV"),
                                COLOR_CLOUD_YUV,
                                None,
                            ),
                        ],
                    ),
                ],
            ),
        )?;

        self.scrollbar.set_range(self.tree.size().1);

        Ok(())
    }

    fn channel_mask_of(&self, id: ControlId) -> ChannelMask {
        match id {
            VIEW_RGB => ChannelMask::new(
                self.tree.window(&VIEW_RGB_R).is_checked(),
                self.tree.window(&VIEW_RGB_G).is_checked(),
                self.tree.window(&VIEW_RGB_B).is_checked(),
            ),
            _ => ChannelMask::new(false, false, false),
        }
    }

    fn update_state(&mut self, id: ControlId) {
        match id {
            VIEW_ORIGINAL => self.state.set_view_mode(ViewMode::Original),
            VIEW_RGB | VIEW_RGB_R | VIEW_RGB_G | VIEW_RGB_B => self
                .state
                .set_view_mode(ViewMode::Rgb(self.channel_mask_of(VIEW_RGB))),
            VIEW_HUE => self.state.set_view_mode(ViewMode::Hue),
            VIEW_SATURATION => self.state.set_view_mode(ViewMode::Saturation),
            VIEW_BRIGHTNESS => self.state.set_view_mode(ViewMode::Brightness),
            HISTOGRAM_DISABLE => self.state.set_histogram_mode(HistogramMode::Disable),
            HISTOGRAM_RGB => self.state.set_histogram_mode(HistogramMode::Rgb),
            HISTOGRAM_HUE => self.state.set_histogram_mode(HistogramMode::Hue),
            HISTOGRAM_SATURAION => self.state.set_histogram_mode(HistogramMode::Saturation),
            HISTOGRAM_BRIGHTNESS => self.state.set_histogram_mode(HistogramMode::Brightness),
            COLOR_CLOUD_DISABLE => self.state.set_color_cloud_mode(ColorCloudMode::Disable),
            COLOR_CLOUD_RGB => self
                .state
                .set_color_cloud_mode(ColorCloudMode::Enable(ColorSpace::Rgb)),
            COLOR_CLOUD_HSV => self
                .state
                .set_color_cloud_mode(ColorCloudMode::Enable(ColorSpace::Hsv)),
            COLOR_CLOUD_HSL => self
                .state
                .set_color_cloud_mode(ColorCloudMode::Enable(ColorSpace::Hsl)),
            COLOR_CLOUD_YUV => self
                .state
                .set_color_cloud_mode(ColorCloudMode::Enable(ColorSpace::Yuv)),
            _ => {}
        }
    }

    fn custom_draw(&self, _wp: WPARAM, lp: LPARAM) -> LRESULT {
        let nmcd: &mut NMCUSTOMDRAW = unsafe { std::mem::transmute(lp) };

        match nmcd.dwDrawStage {
            CDDS_PREPAINT => unsafe {
                let hdc = nmcd.hdc;

                SetTextColor(hdc, Theme::TEXT_COLOR);
                SetBkMode(hdc, TRANSPARENT);

                nmcd.rc.left += self.window.dpi().absolute(17);
                DrawTextA(
                    nmcd.hdc,
                    &mut Window(nmcd.hdr.hwndFrom).text(),
                    &mut nmcd.rc,
                    DT_VCENTER | DT_SINGLELINE,
                );

                LRESULT(CDRF_SKIPDEFAULT as _)
            },
            _ => LRESULT(CDRF_DODEFAULT as _),
        }
    }

    fn on_create(&mut self, hwnd: HWND) -> LRESULT {
        match self.init(hwnd) {
            Ok(_) => LRESULT(0),
            Err(_) => LRESULT(-1),
        }
    }

    fn on_size(&mut self, lp: LPARAM) -> LRESULT {
        let (_x, y) = break_lp(lp);
        self.scrollbar.set_page(y as _);
        LRESULT(0)
    }

    fn on_notify(&mut self, wp: WPARAM, lp: LPARAM) -> LRESULT {
        match unsafe { std::mem::transmute::<_, &NMHDR>(lp) }.code {
            NM_CUSTOMDRAW => self.custom_draw(wp, lp),
            _ => LRESULT(0),
        }
    }

    fn on_command(&mut self, wp: WPARAM) -> LRESULT {
        self.update_state(ControlId::from_wp(wp));
        _ = self.build();
        LRESULT(0)
    }

    fn on_vscroll(&mut self, wp: WPARAM) -> LRESULT {
        self.scrollbar.handle_scroll(wp)
    }

    fn on_mouse_wheel(&mut self, wp: WPARAM) -> LRESULT {
        self.scrollbar.handle_mouse_wheel(wp)
    }

    fn on_ctl_color(&mut self, wp: WPARAM) -> LRESULT {
        unsafe {
            let hdc: HDC = std::mem::transmute(wp);
            SetTextColor(hdc, Theme::TEXT_COLOR);
            SetBkMode(hdc, TRANSPARENT);
        }
        LRESULT(self.theme.brush().0 as _)
    }

    fn handle_message(&mut self, hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> Option<LRESULT> {
        match msg {
            WM_CREATE => Some(self.on_create(hwnd)),
            WM_SIZE => Some(self.on_size(lp)),
            WM_NOTIFY => Some(self.on_notify(wp, lp)),
            WM_COMMAND => Some(self.on_command(wp)),
            WM_VSCROLL => Some(self.on_vscroll(wp)),
            WM_MOUSEWHEEL => Some(self.on_mouse_wheel(wp)),
            WM_CTLCOLORSTATIC => Some(self.on_ctl_color(wp)),
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
            .retrieve_object::<Panel>()
            .and_then(|mut this| this.as_mut().handle_message(hwnd, msg, wp, lp))
            .unwrap_or_else(|| match msg {
                WM_NCCALCSIZE => on_nc_calc_size(hwnd, wp, lp),
                WM_NCHITTEST => on_nc_hit_test(hwnd),
                _ => DefWindowProcA(hwnd, msg, wp, lp),
            })
    }
}

fn on_nc_calc_size(hwnd: HWND, wp: WPARAM, lp: LPARAM) -> LRESULT {
    unsafe {
        let rect: &mut RECT = std::mem::transmute(lp);
        let orig = rect.clone();

        DefWindowProcA(hwnd, WM_NCCALCSIZE, wp, lp);
        rect.left = orig.left;
        rect.top = orig.top;
        rect.bottom = orig.bottom - 1;

        LRESULT(0)
    }
}

fn on_nc_hit_test(hwnd: HWND) -> LRESULT {
    unsafe {
        let lp = {
            let (x, y) = cursor_pos();
            make_lp(x as _, y as _)
        };

        let hit = DefWindowProcA(hwnd, WM_NCHITTEST, WPARAM(0), lp).0 as u32;
        match hit {
            HTVSCROLL | HTNOWHERE => LRESULT(hit as _),
            _ => {
                let hit = Window(hwnd)
                    .owner()
                    .send_message(WM_NCHITTEST, WPARAM(0), lp)
                    .0 as u32;

                match hit {
                    HTCLIENT | HTNOWHERE => LRESULT(hit as _),
                    _ => LRESULT(HTTRANSPARENT as _),
                }
            }
        }
    }
}
