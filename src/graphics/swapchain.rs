use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::{
            Direct3D12::*,
            Dxgi::{Common::*, *},
        },
    },
};

use super::{
    descriptor::{Descriptor, DescriptorHeap},
    device::Device,
    resource::*,
};
use crate::state::compositor::Compositor;
use crate::state::Window;

pub const BUFFER_COUNT: u32 = 2;

pub struct SwapChain {
    pub swapchain: IDXGISwapChain4,
    pub resources: Option<BufferDependentResource>,
}

impl SwapChain {
    pub fn new(
        factory: &IDXGIFactory7,
        device: &Device,
        descriptor_heap: &mut DescriptorHeap,
        command_queue: &ID3D12CommandQueue,
        format: DXGI_FORMAT,
        window: Window,
        compositor: &mut Compositor,
    ) -> Result<Self> {
        unsafe {
            let (width, height) = window.client_size();

            let swapchain1 = factory.CreateSwapChainForComposition(
                command_queue,
                &DXGI_SWAP_CHAIN_DESC1 {
                    Width: width as _,
                    Height: height as _,
                    Format: format,
                    Stereo: FALSE,
                    SampleDesc: SampleDesc::default(),
                    BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                    BufferCount: BUFFER_COUNT,
                    Scaling: DXGI_SCALING_STRETCH,
                    SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                    AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
                    Flags: 0,
                },
                None,
            )?;
            let swapchain: IDXGISwapChain4 = swapchain1.cast()?;
            compositor.bind_swapchain_to_window(&swapchain, window)?;

            let resources = Some(BufferDependentResource::new(
                swapchain.clone(),
                device,
                descriptor_heap,
            )?);

            Ok(Self {
                swapchain,
                resources,
            })
        }
    }

    pub fn present(&self) -> Result<()> {
        unsafe { self.swapchain.Present(1, 0).ok() }
    }

    pub fn resize(
        &mut self,
        device: &Device,
        descriptor_heap: &mut DescriptorHeap,
        width: u32,
        height: u32,
    ) -> Result<()> {
        if let Some(resources) = &self.resources {
            let desc = resources.buffer().desc();
            if desc.Width as u32 == width && desc.Height == height {
                return Ok(());
            }
        }

        self.resources = None;

        unsafe {
            self.swapchain
                .ResizeBuffers(BUFFER_COUNT, width, height, DXGI_FORMAT_UNKNOWN, 0)?;
        }

        self.resources = Some(BufferDependentResource::new(
            self.swapchain.clone(),
            device,
            descriptor_heap,
        )?);

        Ok(())
    }
}

pub struct BufferDependentResource {
    swapchain: IDXGISwapChain4,

    buffers: Vec<Resource>,

    depth: Resource,
}

impl BufferDependentResource {
    pub fn new(
        swapchain: IDXGISwapChain4,
        device: &Device,
        descriptor_heap: &mut DescriptorHeap,
    ) -> Result<Self> {
        unsafe {
            let mut buffers = vec![];
            for i in 0..BUFFER_COUNT {
                let mut buffer = Resource::wrap(swapchain.GetBuffer(i)?);
                descriptor_heap.create_rtv(&mut buffer, i);

                buffers.push(buffer);
            }

            let mut desc = Default::default();
            swapchain.GetDesc(&mut desc)?;

            let mut depth = Resource::new_depth_texture(
                device,
                desc.BufferDesc.Width,
                desc.BufferDesc.Height,
                DXGI_FORMAT_D32_FLOAT,
                D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL | D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE,
            )?;
            descriptor_heap.create_dsv(&mut depth, DXGI_FORMAT_D32_FLOAT, D3D12_DSV_FLAG_NONE);

            Ok(Self {
                swapchain,
                buffers,
                depth,
            })
        }
    }

    pub fn buffer(&self) -> &Resource {
        let index = unsafe { self.swapchain.GetCurrentBackBufferIndex() };
        &self.buffers[index as usize]
    }

    pub fn rtv(&self) -> &Descriptor {
        self.buffer().rtv()
    }

    pub fn dsv(&self) -> &Descriptor {
        self.depth.dsv()
    }
}
