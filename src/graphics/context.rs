use std::cell::RefCell;

use windows::{
    core::*,
    Win32::Graphics::{
        Direct3D12::*,
        Dxgi::{Common::*, *},
    },
};

use crate::{gui::Window, state::compositor::Compositor};

pub use super::{
    command_list::*, compiler::*, descriptor::*, device::*, resource::*, swapchain::SwapChain,
    timer::Timer,
};

pub struct Context {
    pub device: Device,
    pub command_list: CommandList,
    pub swapchain: SwapChain,
    pub descriptor_heap: DescriptorHeap,
    pub compiler: Compiler,
    timer: RefCell<Timer>,
}

impl Context {
    pub fn new(
        factory: &IDXGIFactory7,
        window: Window,
        format: DXGI_FORMAT,
        compositor: &mut Compositor,
    ) -> Result<Self> {
        let device = Device::new()?;

        let mut descriptor_heap = DescriptorHeap::new(device.clone())?;

        let command_list = CommandList::new(&device)?;

        let swapchain = SwapChain::new(
            factory,
            &device,
            &mut descriptor_heap,
            command_list.queue(),
            format,
            window,
            compositor,
        )?;

        let compiler = Compiler::new()?;

        let timer = RefCell::new(Timer::new(&device)?);

        Ok(Self {
            device,
            command_list,
            swapchain,
            descriptor_heap,
            compiler,
            timer,
        })
    }

    pub fn begin_draw(&mut self, width: u32, height: u32, color: &[f32; 4]) -> Result<()> {
        self.swapchain
            .resize(&self.device, &mut self.descriptor_heap, width, height)?;

        if let Some(resources) = &self.swapchain.resources {
            let command_list = &mut self.command_list;

            command_list.reset()?;

            command_list.resource_barrier(&[ResourceBarrier::transition(
                &resources.buffer(),
                D3D12_RESOURCE_STATE_PRESENT,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
            )]);

            let rtv = resources.rtv();
            let dsv = resources.dsv();
            command_list.clear_render_target_view(rtv, color);
            command_list.clear_depth_stencil_view(dsv);

            command_list.set_render_target(rtv, dsv);
            command_list.set_viewport(0.0, 0.0, width as _, height as _);
            command_list.set_scissor_rect(0, 0, width as _, height as _);

            command_list.set_descriptor_heap(&self.descriptor_heap);
        }
        Ok(())
    }

    pub fn end_draw(&mut self) -> Result<()> {
        if let Some(resources) = &self.swapchain.resources {
            let command_list = &mut self.command_list;

            command_list.resource_barrier(&[ResourceBarrier::transition(
                &resources.buffer(),
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                D3D12_RESOURCE_STATE_PRESENT,
            )]);

            self.timer.borrow_mut().resolve(command_list);

            command_list.execute()?;

            self.swapchain.present()?;

            command_list.wait()?;

            self.timer
                .borrow_mut()
                .dump(unsafe { command_list.queue().GetTimestampFrequency()? })?;
        }
        Ok(())
    }

    #[allow(unused)]
    pub fn start_timer(&self, tag: &str) {
        self.timer.borrow_mut().start(&self.command_list, tag);
    }

    #[allow(unused)]
    pub fn stop_timer(&self, tag: &str) {
        self.timer.borrow_mut().stop(&self.command_list, tag);
    }
}
