use super::dpi::Dpi;
use super::Window;
use crate::state::Theme;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use windows::core::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub struct CreateContext {
    dpi: Dpi,
    parent: Window,
    theme: Rc<Theme>,
}

pub trait Key: Copy + PartialEq + Eq + std::hash::Hash + Into<HMENU> + Debug {}

pub trait Widget<K: Key>: Debug {
    fn create(
        &self,
        ctx: &CreateContext,
        x: i32,
        y: i32,
        visible: bool,
        enable: bool,
        cache: &mut HashMap<K, Window>,
    ) -> Result<()>;

    fn update(&self) -> Result<()>;

    fn size(&self, ctx: &CreateContext) -> (i32, i32);
}

#[derive(Debug)]
pub struct Text {
    offset: (i32, i32),
    text: PCSTR,
}

impl Text {
    pub fn new(offset: (i32, i32), text: PCSTR) -> Self {
        Self { offset, text }
    }
}

impl<K: Key> Widget<K> for Text {
    fn create(
        &self,
        ctx: &CreateContext,
        x: i32,
        y: i32,
        _visible: bool,
        _enable: bool,
        _cache: &mut HashMap<K, Window>,
    ) -> Result<()> {
        let (x, y) = (
            x + ctx.dpi.absolute(self.offset.0),
            y + ctx.dpi.absolute(self.offset.1),
        );
        let (cx, cy) = (ctx.dpi.absolute(100), ctx.dpi.absolute(17));

        let window = Window::create(
            s!("STATIC"),
            self.text,
            WINDOW_EX_STYLE(0),
            WS_VISIBLE | WS_CHILD | WS_CLIPSIBLINGS,
            x,
            y,
            cx,
            cy,
            ctx.parent.hwnd(),
            HMENU(0),
            None,
        )?;
        window.set_font(ctx.theme.font());
        window.apply_dark_mode();

        Ok(())
    }

    fn update(&self) -> Result<()> {
        Ok(())
    }

    fn size(&self, ctx: &CreateContext) -> (i32, i32) {
        (
            ctx.dpi.absolute(100 + self.offset.0),
            ctx.dpi.absolute(17 + self.offset.1),
        )
    }
}

#[derive(Debug)]
pub struct Check<K: Key> {
    key: K,
    text: PCSTR,
    checked: bool,
    offset: (i32, i32),
}

impl<K: Key> Check<K> {
    pub fn new(offset: (i32, i32), checked: bool, text: PCSTR, key: K) -> Self {
        Self {
            key,
            text,
            checked,
            offset,
        }
    }
}

impl<K: Key> Widget<K> for Check<K> {
    fn create(
        &self,
        ctx: &CreateContext,
        x: i32,
        y: i32,
        visible: bool,
        enable: bool,
        cache: &mut HashMap<K, Window>,
    ) -> Result<()> {
        let (x, y) = (
            x + ctx.dpi.absolute(self.offset.0),
            y + ctx.dpi.absolute(self.offset.1),
        );
        let (cx, cy) = (ctx.dpi.absolute(17 + 17 * 1), ctx.dpi.absolute(17));

        let window = match cache.entry(self.key) {
            Entry::Occupied(o) => {
                let window = *o.get();
                window.set_position(x, y);
                window
            }
            Entry::Vacant(v) => {
                let window = *v.insert(Window::create(
                    s!("BUTTON"),
                    PCSTR::from_raw(self.text.as_ptr()),
                    WINDOW_EX_STYLE(0),
                    WS_VISIBLE | WS_CHILD | WS_CLIPSIBLINGS | WINDOW_STYLE(BS_AUTOCHECKBOX as _),
                    x,
                    y,
                    cx,
                    cy,
                    ctx.parent.hwnd(),
                    self.key.into(),
                    None,
                )?);
                window.set_font(ctx.theme.font());
                window.apply_dark_mode();
                window.set_check(self.checked);
                window
            }
        };

        window.set_visibility(visible);
        window.set_enabled(enable);

        Ok(())
    }

    fn update(&self) -> Result<()> {
        Ok(())
    }

    fn size(&self, ctx: &CreateContext) -> (i32, i32) {
        (
            ctx.dpi.absolute(17 + 17 * 1 + self.offset.0),
            ctx.dpi.absolute(17 + self.offset.1),
        )
    }
}

#[derive(Debug)]
pub struct Radio<'a, K: Key> {
    key: K,
    text: PCSTR,
    checked: bool,
    group: bool,
    offset: (i32, i32),
    options: Option<&'a dyn Widget<K>>,
}

impl<'a, K: Key> Radio<'a, K> {
    pub fn new(
        offset: (i32, i32),
        checked: bool,
        group: bool,
        text: PCSTR,
        key: K,
        options: Option<&'a dyn Widget<K>>,
    ) -> Self {
        Self {
            key,
            text,
            checked,
            group,
            offset,
            options,
        }
    }
}

impl<'a, K: Key> Widget<K> for Radio<'a, K> {
    fn create(
        &self,
        ctx: &CreateContext,
        x: i32,
        y: i32,
        visible: bool,
        enable: bool,
        cache: &mut HashMap<K, Window>,
    ) -> Result<()> {
        let (x, y) = (
            x + ctx.dpi.absolute(self.offset.0),
            y + ctx.dpi.absolute(self.offset.1),
        );
        let (cx, cy) = (ctx.dpi.absolute(80), ctx.dpi.absolute(17));

        let window = match cache.entry(self.key) {
            Entry::Occupied(o) => {
                let window = *o.get();
                window.set_position(x, y);
                window
            }
            Entry::Vacant(v) => {
                let window = *v.insert(Window::create(
                    s!("BUTTON"),
                    PCSTR::from_raw(self.text.as_ptr()),
                    WINDOW_EX_STYLE(0),
                    WS_VISIBLE
                        | WS_CHILD
                        | WS_CLIPSIBLINGS
                        | WINDOW_STYLE(BS_AUTORADIOBUTTON as _)
                        | if self.group {
                            WS_GROUP
                        } else {
                            WINDOW_STYLE(0)
                        },
                    x,
                    y,
                    cx,
                    cy,
                    ctx.parent.hwnd(),
                    self.key.into(),
                    None,
                )?);
                window.set_font(ctx.theme.font());
                window.apply_dark_mode();
                window.set_check(self.checked);
                window
            }
        };

        window.set_visibility(visible);
        window.set_enabled(enable);

        if let Some(options) = self.options {
            options.create(
                ctx,
                x,
                y + cy,
                visible,
                enable && window.is_checked(),
                cache,
            )?;
        }

        Ok(())
    }

    fn update(&self) -> Result<()> {
        Ok(())
    }

    fn size(&self, ctx: &CreateContext) -> (i32, i32) {
        let (opt_x, opt_y) = match self.options {
            Some(options) => options.size(ctx),
            None => (0, 0),
        };

        (
            ctx.dpi.absolute(80).max(opt_x) + ctx.dpi.absolute(self.offset.0),
            ctx.dpi.absolute(17) + opt_y + ctx.dpi.absolute(self.offset.1),
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Axis {
    Vertical,
    Horizontal,
}

#[derive(Debug)]
pub struct Stack<'a, K: Key> {
    axis: Axis,
    spacing: i32,
    offset: (i32, i32),
    widgets: &'a [&'a dyn Widget<K>],
}

impl<'a, K: Key> Stack<'a, K> {
    pub const fn new(
        offset: (i32, i32),
        axis: Axis,
        spacing: i32,
        widgets: &'a [&'a dyn Widget<K>],
    ) -> Self {
        Self {
            axis,
            spacing,
            offset,
            widgets,
        }
    }
}

impl<'a, K: Key> Widget<K> for Stack<'a, K> {
    fn create(
        &self,
        ctx: &CreateContext,
        x: i32,
        y: i32,
        visible: bool,
        enable: bool,
        cache: &mut HashMap<K, Window>,
    ) -> Result<()> {
        let (mut x, mut y) = (
            x + ctx.dpi.absolute(self.offset.0),
            y + ctx.dpi.absolute(self.offset.1),
        );

        for (i, widget) in self.widgets.iter().enumerate() {
            if i != 0 {
                let spacing = ctx.dpi.absolute(self.spacing);
                match self.axis {
                    Axis::Vertical => y += spacing,
                    Axis::Horizontal => x += spacing,
                }
            }

            widget.create(ctx, x, y, visible, enable, cache)?;

            let (cx, cy) = widget.size(ctx);
            match self.axis {
                Axis::Vertical => y += cy,
                Axis::Horizontal => x += cx,
            }
        }

        Ok(())
    }

    fn update(&self) -> Result<()> {
        Ok(())
    }

    fn size(&self, ctx: &CreateContext) -> (i32, i32) {
        let (mut x, mut y) = (
            ctx.dpi.absolute(self.offset.0),
            ctx.dpi.absolute(self.offset.1),
        );

        for (i, widget) in self.widgets.iter().enumerate() {
            if i != 0 {
                let spacing = ctx.dpi.absolute(self.spacing);
                match self.axis {
                    Axis::Vertical => y += spacing,
                    Axis::Horizontal => x += spacing,
                }
            }

            let (cx, cy) = widget.size(ctx);
            match self.axis {
                Axis::Vertical => {
                    x = x.max(cx);
                    y += cy
                }
                Axis::Horizontal => {
                    x += cx;
                    y = y.max(cy);
                }
            }
        }

        (x, y)
    }
}

pub struct Tree<K: Key> {
    width: i32,
    height: i32,
    cache: HashMap<K, Window>,
}

impl<K: Key> Tree<K> {
    const MARGIN: i32 = 11;

    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            cache: HashMap::new(),
        }
    }

    pub fn view(&mut self, parent: Window, theme: Rc<Theme>, widget: &dyn Widget<K>) -> Result<()> {
        let spos = unsafe { GetScrollPos(parent.hwnd(), SB_VERT) };

        let dpi = parent.dpi();
        let margin = dpi.absolute(Self::MARGIN);
        let mut ctx = CreateContext { dpi, parent, theme };
        widget.create(&mut ctx, margin, margin - spos, true, true, &mut self.cache)?;

        let (cx, cy) = widget.size(&ctx);
        self.width = margin + cx;
        self.height = margin + cy + margin;

        Ok(())
    }

    pub fn size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    pub fn window(&self, key: &K) -> Window {
        self.cache[key]
    }
}
