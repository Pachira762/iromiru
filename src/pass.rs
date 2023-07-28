mod color_cloud_count_pass;
mod color_cloud_indirect_pass;
mod color_cloud_mesh_pass;
mod color_cloud_pass;
mod histogram_pass;
mod view_pass;

use crate::graphics::capture::*;
use crate::graphics::context::*;
use crate::gui::compositor::Compositor;
use crate::state::*;
use windows::core::*;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R8G8B8A8_UNORM;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::System::WinRT::*;

use self::color_cloud_pass::ColorCloudPass;
use self::histogram_pass::HistogramPass;
use self::view_pass::ViewPass;

pub enum RootParam {
    Capture,
    Srvs,
    Uavs,
    Constants,
}

impl From<RootParam> for u32 {
    fn from(value: RootParam) -> Self {
        value as u32
    }
}

pub struct Executor {
    state: RefState,
    window: Window,
    context: Context,
    capturer: Capturer,

    root_signature: ID3D12RootSignature,
    view_pass: ViewPass,
    color_cloud_pass: ColorCloudPass,
    histogram_pass: HistogramPass,
}

impl Executor {
    pub fn new(state: RefState, window: Window, compositor: &mut Compositor) -> Result<Self> {
        unsafe {
            let factory: IDXGIFactory7 = CreateDXGIFactory2(if cfg!(debug_assertions) {
                DXGI_CREATE_FACTORY_DEBUG
            } else {
                0
            })?;
            let capturer = Capturer::new(&factory)?;

            let mut context =
                Context::new(&factory, window, DXGI_FORMAT_R8G8B8A8_UNORM, compositor)?;

            let root_signature = Self::create_root_signature(&mut context)?;
            let view_pass = ViewPass::new(&mut context, &root_signature)?;
            let color_cloud_pass = ColorCloudPass::new(&mut context, &root_signature)?;
            let histogram_pass = HistogramPass::new(&mut context, &root_signature)?;

            Ok(Self {
                state,
                window,
                context,
                capturer,
                root_signature,
                view_pass,
                color_cloud_pass,
                histogram_pass,
            })
        }
    }

    pub fn execute(&mut self) -> Result<()> {
        unsafe { RoInitialize(RO_INIT_MULTITHREADED)? };

        loop {
            let state = self.state.read();

            if !state.active {
                break;
            }

            self.update(state)?;
        }

        Ok(())
    }

    fn update(&mut self, mut state: State) -> Result<()> {
        state.rect = self.window.client_rect();
        state.rect.left = state.rect.left.max(0);
        state.rect.top = state.rect.top.max(0);
        state.rect.right = state.rect.right.min(self.capturer.width() as _);
        state.rect.bottom = state.rect.bottom.min(self.capturer.height() as _);

        let (width, height) = rect_size(&state.rect);
        if width <= 0 || height <= 0 {
            return Ok(());
        }

        let capture = match self.capturer.capture(&mut self.context)? {
            Some(capture) => capture,
            _ => {
                std::thread::sleep(std::time::Duration::from_millis(1));
                return Ok(());
            }
        };

        self.context.begin_draw(
            width as _,
            height as _,
            if state.color_cloud_mode.is_enable() {
                &[0.0, 0.0, 0.0, 0.25]
            } else {
                &[0.0, 0.0, 0.0, 0.0]
            },
        )?;

        self.context
            .command_list
            .set_compute_root_signature(&self.root_signature);

        self.context
            .command_list
            .set_graphics_root_signature(&self.root_signature);

        self.context
            .command_list
            .set_compute_descriptor_table(RootParam::Capture, capture.resource.srv());

        self.context
            .command_list
            .set_graphics_descriptor_table(RootParam::Capture, capture.resource.srv());

        self.view_pass
            .process(&mut self.context, &state, &capture)?;

        self.color_cloud_pass
            .process(&mut self.context, &state, &capture)?;

        self.histogram_pass
            .process(&mut self.context, &state, &capture)?;

        self.context.end_draw()?;

        Ok(())
    }

    fn create_root_signature(context: &mut Context) -> Result<ID3D12RootSignature> {
        let ranges_0 = [D3D12_DESCRIPTOR_RANGE {
            RangeType: D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
            NumDescriptors: 1,
            BaseShaderRegister: 0,
            OffsetInDescriptorsFromTableStart: D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND,
            ..Default::default()
        }];

        let ranges_1 = [D3D12_DESCRIPTOR_RANGE {
            RangeType: D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
            NumDescriptors: 4,
            BaseShaderRegister: 1,
            OffsetInDescriptorsFromTableStart: D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND,
            ..Default::default()
        }];

        let ranges_2 = [D3D12_DESCRIPTOR_RANGE {
            RangeType: D3D12_DESCRIPTOR_RANGE_TYPE_UAV,
            NumDescriptors: 4,
            BaseShaderRegister: 0,
            OffsetInDescriptorsFromTableStart: D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND,
            ..Default::default()
        }];

        let params = [
            D3D12_ROOT_PARAMETER {
                ParameterType: D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
                Anonymous: D3D12_ROOT_PARAMETER_0 {
                    DescriptorTable: D3D12_ROOT_DESCRIPTOR_TABLE {
                        NumDescriptorRanges: ranges_0.len() as _,
                        pDescriptorRanges: ranges_0.as_ptr(),
                    },
                },
                ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
            },
            D3D12_ROOT_PARAMETER {
                ParameterType: D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
                Anonymous: D3D12_ROOT_PARAMETER_0 {
                    DescriptorTable: D3D12_ROOT_DESCRIPTOR_TABLE {
                        NumDescriptorRanges: ranges_1.len() as _,
                        pDescriptorRanges: ranges_1.as_ptr(),
                    },
                },
                ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
            },
            D3D12_ROOT_PARAMETER {
                ParameterType: D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
                Anonymous: D3D12_ROOT_PARAMETER_0 {
                    DescriptorTable: D3D12_ROOT_DESCRIPTOR_TABLE {
                        NumDescriptorRanges: ranges_2.len() as _,
                        pDescriptorRanges: ranges_2.as_ptr(),
                    },
                },
                ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
            },
            D3D12_ROOT_PARAMETER {
                ParameterType: D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS,
                Anonymous: D3D12_ROOT_PARAMETER_0 {
                    Constants: D3D12_ROOT_CONSTANTS {
                        ShaderRegister: 0,
                        Num32BitValues: 29,
                        ..Default::default()
                    },
                },
                ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
            },
        ];

        let sampler = [D3D12_STATIC_SAMPLER_DESC {
            Filter: D3D12_FILTER_MIN_MAG_MIP_POINT,
            AddressU: D3D12_TEXTURE_ADDRESS_MODE_WRAP,
            AddressV: D3D12_TEXTURE_ADDRESS_MODE_WRAP,
            AddressW: D3D12_TEXTURE_ADDRESS_MODE_WRAP,
            MipLODBias: 0.0,
            MaxAnisotropy: 0,
            ComparisonFunc: D3D12_COMPARISON_FUNC_LESS_EQUAL,
            BorderColor: D3D12_STATIC_BORDER_COLOR_OPAQUE_WHITE,
            MinLOD: 0.0,
            MaxLOD: D3D12_FLOAT32_MAX,
            ShaderRegister: 0,
            RegisterSpace: 0,
            ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
        }];

        context.device.create_root_signature(
            &params,
            &sampler,
            D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
        )
    }
}

unsafe impl Send for Executor {}
