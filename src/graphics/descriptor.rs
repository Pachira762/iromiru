use windows::core::*;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::*;

use super::context::Resource;
use super::device::Device;
use super::swapchain::BUFFER_COUNT;

const MAX_DESCRIPTORS: u32 = 64;

#[derive(Clone, Copy, Default, Debug)]
pub struct Descriptor {
    pub cpu: D3D12_CPU_DESCRIPTOR_HANDLE,
    pub gpu: D3D12_GPU_DESCRIPTOR_HANDLE,
}

pub struct DescriptorHeap {
    device: Device,
    pub shader_visible_heap: ID3D12DescriptorHeap,
    pub non_shader_visible_heap: ID3D12DescriptorHeap,
    rtv_heap: ID3D12DescriptorHeap,
    dsv_heap: ID3D12DescriptorHeap,
    srv_size: u32,
    rtv_size: u32,
    dsv_size: u32,
    num_shader_visibles: u32,
    num_non_shader_visibles: u32,
}

impl DescriptorHeap {
    pub fn new(device: Device) -> Result<Self> {
        let shader_visible_heap = device.create_descriptor_heap(
            D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
            MAX_DESCRIPTORS,
            D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
        )?;

        let non_shader_visible_heap = device.create_descriptor_heap(
            D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
            MAX_DESCRIPTORS,
            D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
        )?;

        let rtv_heap = device.create_descriptor_heap(
            D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            BUFFER_COUNT,
            D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
        )?;

        let dsv_heap = device.create_descriptor_heap(
            D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
            1,
            D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
        )?;

        let srv_size = device.descriptor_size(D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV);
        let rtv_size = device.descriptor_size(D3D12_DESCRIPTOR_HEAP_TYPE_RTV);
        let dsv_size = device.descriptor_size(D3D12_DESCRIPTOR_HEAP_TYPE_DSV);

        Ok(Self {
            device,
            shader_visible_heap,
            non_shader_visible_heap,
            rtv_heap,
            dsv_heap,
            srv_size,
            rtv_size,
            dsv_size,
            num_shader_visibles: 1, // 0 for capture srv
            num_non_shader_visibles: 0,
        })
    }

    pub fn heap(&self) -> &ID3D12DescriptorHeap {
        &self.shader_visible_heap
    }

    pub fn create_srv_at(
        &mut self,
        index: u32,
        resource: &mut Resource,
        desc: Option<*const D3D12_SHADER_RESOURCE_VIEW_DESC>,
    ) {
        let descriptor = self.descriptor(ShaderVisible, index);

        self.device
            .create_shader_resource_view(resource, desc, descriptor.cpu);

        resource.set_srv(descriptor);
    }

    pub fn create_srv(
        &mut self,
        resource: &mut Resource,
        desc: Option<*const D3D12_SHADER_RESOURCE_VIEW_DESC>,
    ) {
        let descriptor = self.descriptor(ShaderVisible, self.num_shader_visibles);
        self.num_shader_visibles += 1;

        self.device
            .create_shader_resource_view(resource, desc, descriptor.cpu);

        resource.set_srv(descriptor);
    }

    pub fn create_srv_buffer(
        &mut self,
        resource: &mut Resource,
        format: Option<DXGI_FORMAT>,
        stride: Option<u32>,
        num: u32,
    ) {
        let (format, stride, flags) = match (format, stride) {
            (Some(format), _) => (format, 0, D3D12_BUFFER_SRV_FLAG_NONE),
            (None, Some(stride)) => (DXGI_FORMAT_UNKNOWN, stride, D3D12_BUFFER_SRV_FLAG_NONE),
            (None, None) => (DXGI_FORMAT_R32_TYPELESS, 0, D3D12_BUFFER_SRV_FLAG_RAW),
        };

        self.create_srv(
            resource,
            Some(&D3D12_SHADER_RESOURCE_VIEW_DESC {
                Format: format,
                ViewDimension: D3D12_SRV_DIMENSION_BUFFER,
                Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
                Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                    Buffer: D3D12_BUFFER_SRV {
                        FirstElement: 0,
                        NumElements: num,
                        StructureByteStride: stride,
                        Flags: flags,
                    },
                },
            }),
        );
    }

    #[allow(unused)]
    pub fn create_srv_tex2d(&mut self, resource: &mut Resource) {
        self.create_srv(resource, None)
    }

    pub fn create_uav(
        &mut self,
        resource: &mut Resource,
        counter: bool,
        desc: &D3D12_UNORDERED_ACCESS_VIEW_DESC,
    ) {
        let descriptor = self.descriptor(ShaderVisible, self.num_shader_visibles);
        self.num_shader_visibles += 1;

        self.device.create_unordered_access_view(
            resource,
            if counter { Some(resource) } else { None },
            Some(desc),
            descriptor.cpu,
        );

        resource.set_uav(descriptor);
    }

    pub fn create_uav_to_clear(&mut self, resource: &mut Resource, num: u32, offset: u64) {
        let shader_visible = self.descriptor(ShaderVisible, self.num_shader_visibles);
        self.num_shader_visibles += 1;

        let non_shader_visible = self.descriptor(NonShaderVisible, self.num_non_shader_visibles);
        self.num_non_shader_visibles += 1;

        let desc = D3D12_UNORDERED_ACCESS_VIEW_DESC {
            Format: DXGI_FORMAT_R32_TYPELESS,
            ViewDimension: D3D12_UAV_DIMENSION_BUFFER,
            Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
                Buffer: D3D12_BUFFER_UAV {
                    FirstElement: offset,
                    NumElements: num,
                    StructureByteStride: 0,
                    CounterOffsetInBytes: 0,
                    Flags: D3D12_BUFFER_UAV_FLAG_RAW,
                },
            },
        };

        self.device
            .create_unordered_access_view(resource, None, Some(&desc), shader_visible.cpu);

        self.device.create_unordered_access_view(
            resource,
            None,
            Some(&desc),
            non_shader_visible.cpu,
        );

        resource.set_uav_to_clear(shader_visible, non_shader_visible);
    }

    pub fn create_uav_buffer(
        &mut self,
        resource: &mut Resource,
        format: Option<DXGI_FORMAT>,
        stride: Option<u32>,
        num: u32,
        first_elem: Option<u64>,
        counter_offset: Option<u64>,
    ) {
        let (counter, offset) = match counter_offset {
            Some(offset) => (true, offset),
            _ => (false, 0),
        };

        let (format, stride, flags) = match (format, stride) {
            (Some(format), _) => (format, 0, D3D12_BUFFER_UAV_FLAG_NONE),
            (None, Some(stride)) => (DXGI_FORMAT_UNKNOWN, stride, D3D12_BUFFER_UAV_FLAG_NONE),
            (None, None) => (DXGI_FORMAT_R32_TYPELESS, 0, D3D12_BUFFER_UAV_FLAG_RAW),
        };

        self.create_uav(
            resource,
            counter,
            &D3D12_UNORDERED_ACCESS_VIEW_DESC {
                Format: format,
                ViewDimension: D3D12_UAV_DIMENSION_BUFFER,
                Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
                    Buffer: D3D12_BUFFER_UAV {
                        FirstElement: first_elem.unwrap_or_default(),
                        NumElements: num,
                        StructureByteStride: stride,
                        CounterOffsetInBytes: offset,
                        Flags: flags,
                    },
                },
            },
        );
    }

    pub fn create_rtv(&mut self, resource: &mut Resource, index: u32) {
        let descriptor = self.descriptor(Rtv, index);

        self.device
            .create_render_target_view(resource, None, descriptor.cpu);

        resource.set_rtv(descriptor);
    }

    pub fn create_dsv(
        &mut self,
        resource: &mut Resource,
        format: DXGI_FORMAT,
        flags: D3D12_DSV_FLAGS,
    ) {
        let descriptor = self.descriptor(Dsv, 0);

        self.device.create_depth_stencil_view(
            resource,
            Some(&D3D12_DEPTH_STENCIL_VIEW_DESC {
                Format: format,
                ViewDimension: D3D12_DSV_DIMENSION_TEXTURE2D,
                Flags: flags,
                Anonymous: D3D12_DEPTH_STENCIL_VIEW_DESC_0 {
                    Texture2D: D3D12_TEX2D_DSV { MipSlice: 0 },
                },
            }),
            descriptor.cpu,
        );

        resource.set_dsv(descriptor);
    }

    fn descriptor(&mut self, kind: Kind, index: u32) -> Descriptor {
        unsafe {
            let heap = self.heap_of(kind);

            let mut cpu = heap.GetCPUDescriptorHandleForHeapStart();
            cpu.ptr += (index * self.size_of(kind)) as usize;

            let gpu = if kind == ShaderVisible {
                let mut gpu = heap.GetGPUDescriptorHandleForHeapStart();
                gpu.ptr += (index * self.size_of(kind)) as u64;
                gpu
            } else {
                Default::default()
            };

            Descriptor { cpu, gpu }
        }
    }

    fn heap_of(&self, kind: Kind) -> &ID3D12DescriptorHeap {
        match kind {
            ShaderVisible => &self.shader_visible_heap,
            NonShaderVisible => &self.non_shader_visible_heap,
            Rtv => &self.rtv_heap,
            Dsv => &self.dsv_heap,
        }
    }

    fn size_of(&self, kind: Kind) -> u32 {
        match kind {
            ShaderVisible | NonShaderVisible => self.srv_size,
            Rtv => self.rtv_size,
            Dsv => self.dsv_size,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    ShaderVisible,
    NonShaderVisible,
    Rtv,
    Dsv,
}
use Kind::*;
