use crate::graphics::{capture::Capture, context::*};
use crate::state::*;
use glam::*;
use std::mem::size_of;
use windows::core::*;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct3D::Dxc::DxcDefine;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::{w, Win32::Graphics::Direct3D12::*};

use super::RootParam;

pub struct ColorCloudCountPass {
    count_pso: ID3D12PipelineState,
    count_buf: Resource,
}

impl ColorCloudCountPass {
    pub fn new(context: &mut Context, root_signature: &ID3D12RootSignature) -> Result<Self> {
        let device = &context.device;
        let descriptor_heap = &mut context.descriptor_heap;
        let compiler = &context.compiler;

        let count_pso = device.create_compute_pipeline(
            &root_signature,
            &compiler.compile(
                w!("shaders\\color_cloud.hlsl"),
                w!("CountCs"),
                w!("cs_6_5"),
                &[
                    DxcDefine {
                        Name: w!("COUNT"),
                        Value: w!(""),
                    },
                    DxcDefine {
                        Name: w!("CS_6_5"),
                        Value: w!(""),
                    },
                ],
            )?,
        )?;

        let mut count_buf = Resource::new_buffer(
            &device,
            4 * 256 * 256 * 256,
            D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            D3D12_RESOURCE_STATE_COMMON,
        )?;

        descriptor_heap.create_srv_buffer(
            &mut count_buf,
            Some(DXGI_FORMAT_R32_UINT),
            None,
            256 * 256 * 256,
        );

        descriptor_heap.create_uav_buffer(
            &mut count_buf,
            Some(DXGI_FORMAT_R32_UINT),
            None,
            256 * 256 * 256,
            None,
            None,
        );

        descriptor_heap.create_uav_to_clear(&mut count_buf, 256 * 256 * 256, 0);

        Ok(Self {
            count_pso,
            count_buf,
        })
    }

    pub fn process(
        &mut self,
        context: &mut Context,
        state: &State,
        _capture: &Capture,
    ) -> Result<()> {
        if state.color_cloud_mode.is_enable() {
            self.clear(context)?;
            self.count(context, state)?;
            self.transition(context)?;
        }
        Ok(())
    }

    #[allow(unused)]
    pub fn dump(&self) -> Result<()> {
        unsafe { Ok(()) }
    }

    fn clear(&mut self, context: &mut Context) -> Result<()> {
        let command_list = &context.command_list;

        command_list.resource_barrier(&[ResourceBarrier::transition(
            &self.count_buf,
            D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
        )]);

        command_list.clear_unordered_access_view_uint(&self.count_buf, &[0, 0, 0, 0], &[]);

        Ok(())
    }

    fn count(&mut self, context: &mut Context, state: &State) -> Result<()> {
        #[repr(C)]
        struct Params {
            rect: RECT,
        }

        let rect = state.rect;
        let width = rect_width(&rect) as u32;
        let height = rect_height(&rect) as u32;

        const THREAD: u32 = 8;
        let dim_x = div_round_up(width, THREAD);
        let dim_y = div_round_up(height, THREAD);

        let command_list = &context.command_list;
        command_list.set_pipeline_state(&self.count_pso);

        command_list.set_compute_constants(
            RootParam::Constants,
            size_of::<Params>() as u32 / 4,
            &Params { rect: state.rect } as *const _ as _,
        );

        command_list.set_compute_descriptor_table(RootParam::Uavs, self.count_buf.uav());

        command_list.dispatch(dim_x, dim_y, 1);

        Ok(())
    }

    fn transition(&mut self, context: &mut Context) -> Result<()> {
        let command_list = &context.command_list;

        command_list.resource_barrier(&[ResourceBarrier::transition(
            &self.count_buf,
            D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
        )]);

        command_list.set_compute_descriptor_table(RootParam::Srvs, self.count_buf.srv());
        command_list.set_graphics_descriptor_table(RootParam::Srvs, self.count_buf.srv());

        Ok(())
    }
}
