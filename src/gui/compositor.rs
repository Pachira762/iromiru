use crate::gui::Window;
use windows::core::*;
use windows::Foundation::Numerics::Vector2;
use windows::Win32::System::WinRT::Composition::*;
use windows::UI::Composition::Desktop::DesktopWindowTarget;

pub struct Compositor {
    #[allow(unused)]
    compositor: windows::UI::Composition::Compositor,

    #[allow(unused)]
    window_targets: Vec<DesktopWindowTarget>,
}

impl Compositor {
    pub fn new() -> Result<Self> {
        let compositor = windows::UI::Composition::Compositor::new()?;

        Ok(Self {
            compositor,
            window_targets: vec![],
        })
    }

    pub fn bind_swapchain_to_window<P: windows::core::IntoParam<IUnknown>>(
        &mut self,
        swapchain: P,
        window: Window,
    ) -> Result<()> {
        let surface = unsafe {
            let interpo: ICompositorInterop = self.compositor.cast()?;
            interpo.CreateCompositionSurfaceForSwapChain(swapchain)
        }?;

        let brush = self.compositor.CreateSurfaceBrushWithSurface(&surface)?;

        let content = self.compositor.CreateSpriteVisual()?;
        content.SetRelativeSizeAdjustment(Vector2::new(1.0, 1.0))?;
        content.SetBrush(&brush)?;

        let target = unsafe {
            let interpo: ICompositorDesktopInterop = self.compositor.cast()?;
            interpo.CreateDesktopWindowTarget(window.hwnd(), true)
        }?;
        target.SetRoot(&content)?;

        self.window_targets.push(target);

        Ok(())
    }
}
