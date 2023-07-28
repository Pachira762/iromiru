use std::mem::size_of;
use std::ops::Deref;
use std::os::raw::c_void;

use windows::core::*;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::*;

use super::descriptor::*;
use super::device::*;

pub struct Resource {
    pub resource: ID3D12Resource,
    srv: Descriptor,
    uav: Descriptor,
    misc0: Descriptor,
    misc1: Descriptor,
}

impl Resource {
    pub fn new(
        device: &Device,
        heap_props: D3D12_HEAP_PROPERTIES,
        heap_flags: D3D12_HEAP_FLAGS,
        desc: D3D12_RESOURCE_DESC,
        state: D3D12_RESOURCE_STATES,
        clear: Option<*const D3D12_CLEAR_VALUE>,
    ) -> Result<Self> {
        Ok(Self {
            resource: device.create_resource(&heap_props, heap_flags, &desc, state, clear)?,
            srv: Descriptor::default(),
            uav: Descriptor::default(),
            misc0: Descriptor::default(),
            misc1: Descriptor::default(),
        })
    }

    #[allow(unused)]
    pub fn new_buffer(
        device: &Device,
        size: u64,
        flags: D3D12_RESOURCE_FLAGS,
        state: D3D12_RESOURCE_STATES,
    ) -> Result<Self> {
        Self::new(
            device,
            HeapProps::default(),
            D3D12_HEAP_FLAG_NONE,
            ResourceDesc::buffer(size, flags),
            state,
            None,
        )
    }

    #[allow(unused)]
    pub fn new_upload_buffer(device: &Device, size: u64) -> Result<Self> {
        Self::new(
            device,
            HeapProps::upload(),
            D3D12_HEAP_FLAG_NONE,
            ResourceDesc::buffer(size, D3D12_RESOURCE_FLAG_NONE),
            D3D12_RESOURCE_STATE_COPY_DEST,
            None,
        )
    }

    #[allow(unused)]
    pub fn new_staging_buffer(device: &Device, size: u64) -> Result<Self> {
        Self::new(
            device,
            HeapProps::readback(),
            D3D12_HEAP_FLAG_NONE,
            ResourceDesc::buffer(size, D3D12_RESOURCE_FLAG_NONE),
            D3D12_RESOURCE_STATE_COPY_DEST,
            None,
        )
    }

    #[allow(unused)]
    pub fn new_texture2d(
        device: &Device,
        width: u32,
        height: u32,
        format: DXGI_FORMAT,
        sample_desc: DXGI_SAMPLE_DESC,
        flags: D3D12_RESOURCE_FLAGS,
        state: D3D12_RESOURCE_STATES,
    ) -> Result<Self> {
        Self::new(
            device,
            HeapProps::default(),
            D3D12_HEAP_FLAG_NONE,
            ResourceDesc::texture2d(width, height, format, flags),
            state,
            None,
        )
    }

    #[allow(unused)]
    pub fn new_depth_texture(
        device: &Device,
        width: u32,
        height: u32,
        format: DXGI_FORMAT,
        flags: D3D12_RESOURCE_FLAGS,
    ) -> Result<Self> {
        Self::new(
            device,
            HeapProps::default(),
            D3D12_HEAP_FLAG_NONE,
            ResourceDesc::texture2d(width, height, format, flags),
            D3D12_RESOURCE_STATE_DEPTH_WRITE,
            Some(&ClearValue::depth(format)),
        )
    }

    pub fn from_handle(device: &Device, handle: HANDLE) -> Result<Self> {
        Ok(Self::wrap(device.open_shared_handle(handle)?))
    }

    pub fn wrap(resource: ID3D12Resource) -> Self {
        Self {
            resource,
            srv: Descriptor::default(),
            uav: Descriptor::default(),
            misc0: Descriptor::default(),
            misc1: Descriptor::default(),
        }
    }

    pub fn srv(&self) -> &Descriptor {
        &self.srv
    }

    pub fn set_srv(&mut self, descriptor: Descriptor) {
        self.srv = descriptor;
    }

    pub fn uav(&self) -> &Descriptor {
        &self.uav
    }

    pub fn set_uav(&mut self, descriptor: Descriptor) {
        self.uav = descriptor;
    }

    pub fn uav_to_clear(&self) -> (Descriptor, Descriptor) {
        (self.misc0, self.misc1)
    }

    pub fn set_uav_to_clear(&mut self, shader_visible: Descriptor, non_shader_visible: Descriptor) {
        self.misc0 = shader_visible;
        self.misc1 = non_shader_visible;
    }

    pub fn rtv(&self) -> &Descriptor {
        &self.misc0
    }

    pub fn set_rtv(&mut self, descriptor: Descriptor) {
        self.misc0 = descriptor;
    }

    pub fn dsv(&self) -> &Descriptor {
        &self.misc1
    }

    pub fn set_dsv(&mut self, descriptor: Descriptor) {
        self.misc1 = descriptor;
    }

    pub fn read<T: Clone>(&self, len: usize) -> Result<Vec<T>> {
        unsafe {
            let mut data: *mut T = std::ptr::null_mut();

            self.resource.Map(0, None, Some(&mut data as *mut _ as _))?;
            let data = std::slice::from_raw_parts(data, len).to_vec();
            self.resource.Unmap(0, None);

            Ok(data)
        }
    }

    #[allow(unused)]
    pub fn write<T>(&self, src: *const T, len: usize) -> Result<()> {
        unsafe {
            let range = D3D12_RANGE {
                Begin: 0,
                End: size_of::<T>() * len,
            };
            let mut dest: *mut c_void = std::ptr::null_mut();

            self.resource.Map(0, Some(&range), Some(&mut dest))?;
            std::ptr::copy_nonoverlapping(src, dest as *mut _, len);
            self.resource.Unmap(0, Some(&range));
        }
        Ok(())
    }

    #[allow(unused)]
    pub fn desc(&self) -> D3D12_RESOURCE_DESC {
        unsafe { self.resource.GetDesc() }
    }
}

impl Deref for Resource {
    type Target = ID3D12Resource;

    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}

pub struct HeapProps();

impl HeapProps {
    pub fn default() -> D3D12_HEAP_PROPERTIES {
        D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_DEFAULT,
            CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        }
    }

    pub fn upload() -> D3D12_HEAP_PROPERTIES {
        D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_UPLOAD,
            CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        }
    }

    pub fn readback() -> D3D12_HEAP_PROPERTIES {
        D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_READBACK,
            CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        }
    }

    #[allow(unused)]
    pub fn custom() -> D3D12_HEAP_PROPERTIES {
        D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_CUSTOM,
            CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        }
    }
}

pub struct ResourceDesc();

impl ResourceDesc {
    pub fn buffer(size: u64, flags: D3D12_RESOURCE_FLAGS) -> D3D12_RESOURCE_DESC {
        D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
            Alignment: 0,
            Width: size,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: DXGI_FORMAT_UNKNOWN,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            Flags: flags,
        }
    }

    pub fn texture2d(
        width: u32,
        height: u32,
        format: DXGI_FORMAT,
        flags: D3D12_RESOURCE_FLAGS,
    ) -> D3D12_RESOURCE_DESC {
        D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
            Alignment: 0,
            Width: width as _,
            Height: height,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: format,
            SampleDesc: SampleDesc::default(),
            Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
            Flags: flags,
        }
    }
}

pub struct ClearValue();

impl ClearValue {
    #[allow(unused)]
    pub fn render_target(format: DXGI_FORMAT, color: [f32; 4]) -> D3D12_CLEAR_VALUE {
        D3D12_CLEAR_VALUE {
            Format: format,
            Anonymous: D3D12_CLEAR_VALUE_0 { Color: color },
        }
    }

    pub fn depth(format: DXGI_FORMAT) -> D3D12_CLEAR_VALUE {
        D3D12_CLEAR_VALUE {
            Format: format,
            Anonymous: D3D12_CLEAR_VALUE_0 {
                DepthStencil: D3D12_DEPTH_STENCIL_VALUE {
                    Depth: 1.0,
                    Stencil: 0,
                },
            },
        }
    }
}

pub struct SampleDesc();

impl SampleDesc {
    pub fn default() -> DXGI_SAMPLE_DESC {
        DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        }
    }

    #[allow(unused)]
    pub fn msaa(count: u32) -> DXGI_SAMPLE_DESC {
        DXGI_SAMPLE_DESC {
            Count: count,
            Quality: 0xffffffff,
        }
    }
}
