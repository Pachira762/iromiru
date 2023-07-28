use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;

use super::context::Context;
use super::resource::Resource;

pub struct Capturer {
    dupl: IDXGIOutputDuplication,
    desc: DXGI_OUTDUPL_DESC,
    captured: bool,
    handle: HANDLE,
}

impl Capturer {
    pub fn new(factory: &IDXGIFactory7) -> Result<Self> {
        unsafe {
            let adapter: IDXGIAdapter4 = factory.EnumAdapters1(0)?.cast()?;

            let mut device = None;
            let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT
                | if cfg!(debug_assertions) {
                    D3D11_CREATE_DEVICE_DEBUG
                } else {
                    D3D11_CREATE_DEVICE_FLAG(0)
                };
            D3D11CreateDevice(
                &adapter,
                D3D_DRIVER_TYPE_UNKNOWN,
                None,
                flags,
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                None,
            )?;
            let device = device.unwrap();

            let output: IDXGIOutput6 = adapter.EnumOutputs(0)?.cast()?;
            let dupl = output.DuplicateOutput1(
                &device,
                0,
                &[DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R16G16B16A16_FLOAT],
            )?;

            let desc = {
                let mut desc = Default::default();
                dupl.GetDesc(&mut desc);
                desc
            };

            Ok(Self {
                dupl,
                desc,
                captured: false,
                handle: HANDLE(0),
            })
        }
    }

    pub fn capture(&mut self, context: &mut Context) -> Result<Option<Capture>> {
        unsafe {
            if self.handle != HANDLE(0) {
                CloseHandle(self.handle);
                self.handle = HANDLE(0);
            }

            if self.captured {
                self.dupl.ReleaseFrame()?;
                self.captured = false;
            }

            let mut resource = None;
            let mut info = DXGI_OUTDUPL_FRAME_INFO::default();

            if let Err(e) = self.dupl.AcquireNextFrame(1000, &mut info, &mut resource) {
                if e.code() == DXGI_ERROR_WAIT_TIMEOUT {
                    return Ok(None);
                } else {
                    return Err(e);
                }
            }

            self.captured = true;

            if info.AccumulatedFrames == 0 {
                return Ok(None);
            }

            let resource: IDXGIResource1 = resource.unwrap().cast()?;
            self.handle = resource.CreateSharedHandle(None, DXGI_SHARED_RESOURCE_READ, None)?;

            Ok(Some(Capture::new(context, self.handle)?))
        }
    }

    pub fn width(&self) -> u32 {
        self.desc.ModeDesc.Width
    }

    pub fn height(&self) -> u32 {
        self.desc.ModeDesc.Height
    }
}

pub struct Capture {
    pub resource: Resource,
}

impl Capture {
    fn new(context: &mut Context, handle: HANDLE) -> Result<Self> {
        let mut resource = Resource::from_handle(&context.device, handle)?;

        context
            .descriptor_heap
            .create_srv_at(0, &mut resource, None);

        Ok(Self { resource })
    }
}
