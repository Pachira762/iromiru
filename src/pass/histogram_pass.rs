use std::mem::size_of;

use windows::{
    core::*,
    w,
    Win32::{
        Foundation::RECT,
        Graphics::{
            Direct3D::{Dxc::DxcDefine, *},
            Direct3D12::*,
            Dxgi::Common::*,
        },
    },
};

use crate::{
    graphics::{capture::Capture, *},
    state::*,
};

use super::RootParam;

pub struct HistogramPass {
    create_pso: ID3D12PipelineState,
    fill_pso: ID3D12PipelineState,
    line_pso: ID3D12PipelineState,

    buffers: [Resource; 3],
}

impl HistogramPass {
    pub fn new(context: &mut Context, root_signature: &ID3D12RootSignature) -> Result<Self> {
        let device = &context.device;
        let compiler = &context.compiler;
        let descriptor_heap = &mut context.descriptor_heap;

        let create_pso = device.create_compute_pipeline(
            root_signature,
            &compiler.compile(
                w!("shaders\\histogram.hlsl"),
                w!("CreateCs"),
                w!("cs_6_5"),
                &[DxcDefine {
                    Name: w!("CREATE"),
                    Value: w!(""),
                }],
            )?,
        )?;

        let fill_pso = device.create_graphics_pipeline(
            root_signature,
            &compiler.compile(
                w!("shaders\\histogram.hlsl"),
                w!("FillVs"),
                w!("vs_6_0"),
                &[DxcDefine {
                    Name: w!("DRAW"),
                    Value: w!(""),
                }],
            )?,
            &compiler.compile(
                w!("shaders\\histogram.hlsl"),
                w!("DrawPs"),
                w!("ps_6_0"),
                &[DxcDefine {
                    Name: w!("DRAW"),
                    Value: w!(""),
                }],
            )?,
            BlendState::alpha(),
            RasterizerState::no_cull(),
            DepthStencilState::none(),
            &[],
            None,
            None,
            None,
            None,
        )?;

        let line_pso = device.create_graphics_pipeline(
            root_signature,
            &compiler.compile(
                w!("shaders\\histogram.hlsl"),
                w!("LineVs"),
                w!("vs_6_0"),
                &[DxcDefine {
                    Name: w!("DRAW"),
                    Value: w!(""),
                }],
            )?,
            &compiler.compile(
                w!("shaders\\histogram.hlsl"),
                w!("DrawPs"),
                w!("ps_6_0"),
                &[DxcDefine {
                    Name: w!("DRAW"),
                    Value: w!(""),
                }],
            )?,
            BlendState::alpha(),
            RasterizerState::no_cull(),
            DepthStencilState::none(),
            &[],
            None,
            None,
            None,
            None,
        )?;

        let mut buffers = [
            Resource::new_buffer(
                device,
                4 * 256,
                D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_COMMON,
            )?,
            Resource::new_buffer(
                device,
                4 * 256,
                D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_COMMON,
            )?,
            Resource::new_buffer(
                device,
                4 * 256,
                D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_COMMON,
            )?,
        ];

        for buffer in &mut buffers {
            descriptor_heap.create_srv_buffer(buffer, Some(DXGI_FORMAT_R32_UINT), None, 256);
        }

        for buffer in &mut buffers {
            descriptor_heap.create_uav_buffer(
                buffer,
                Some(DXGI_FORMAT_R32_UINT),
                None,
                256,
                None,
                None,
            );
        }

        for buffer in &mut buffers {
            descriptor_heap.create_uav_to_clear(buffer, 256, 0);
        }

        Ok(Self {
            create_pso,
            fill_pso,
            line_pso,
            buffers,
        })
    }

    pub fn process(
        &mut self,
        context: &mut Context,
        state: &State,
        _capture: &Capture,
    ) -> Result<()> {
        if state.histogram_mode != HistogramMode::Disable {
            self.clear(context)?;
            self.create(context, state)?;
            self.draw(context, state)?;
        }
        Ok(())
    }

    fn clear(&mut self, context: &mut Context) -> Result<()> {
        let command_list = &context.command_list;

        command_list.resource_barrier(&[
            ResourceBarrier::transition(
                &self.buffers[0],
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            ),
            ResourceBarrier::transition(
                &self.buffers[1],
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            ),
            ResourceBarrier::transition(
                &self.buffers[2],
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            ),
        ]);

        for buffer in &self.buffers {
            command_list.clear_unordered_access_view_uint(buffer, &[0; 4], &[]);
        }

        Ok(())
    }

    fn create(&mut self, context: &mut Context, state: &State) -> Result<()> {
        #[repr(C)]
        struct Params {
            rect: RECT,
            mode: u32,
        }
        const NUM_CONSTS: u32 = size_of::<Params>() as u32 / 4;

        let command_list = &context.command_list;

        command_list.set_pipeline_state(&self.create_pso);

        command_list.set_compute_constants(
            RootParam::Constants,
            NUM_CONSTS,
            &Params {
                rect: state.rect,
                mode: state.histogram_mode as _,
            } as *const _ as _,
        );

        command_list.set_compute_descriptor_table(RootParam::Uavs, self.buffers[0].uav());

        const THREADS: u32 = 8;
        let (width, height) = rect_size(&state.rect);

        command_list.dispatch(
            div_round_up(width as _, THREADS),
            div_round_up(height as _, THREADS),
            1,
        );

        Ok(())
    }

    fn draw(&mut self, context: &mut Context, state: &State) -> Result<()> {
        #[repr(C)]
        struct Params {
            color: [f32; 4],
            scale: [f32; 2],
            inv_pixel_count: f32,
            mode: u32,
        }
        const NUM_CONSTS: u32 = size_of::<Params>() as u32 / 4;

        let command_list = &context.command_list;

        command_list.resource_barrier(&[
            ResourceBarrier::transition(
                &self.buffers[0],
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            ),
            ResourceBarrier::transition(
                &self.buffers[1],
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            ),
            ResourceBarrier::transition(
                &self.buffers[2],
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            ),
        ]);

        let (width, height) = rect_size(&state.rect);
        let mut params = Params {
            color: [0.0; 4],
            scale: [(height as f32) / (width as f32), 1.0],
            inv_pixel_count: 4.0 / ((width * height) as f32),
            mode: state.histogram_mode as _,
        };

        if state.histogram_mode == HistogramMode::Rgb {
            const FILL_COLORS: [[f32; 4]; 3] = [
                [0.5, 0.0, 0.0, 0.6],
                [0.0, 0.5, 0.0, 0.6],
                [0.0, 0.0, 0.5, 0.6],
            ];

            const LINE_COLORS: [[f32; 4]; 3] = [
                [0.8, 0.0, 0.0, 0.8],
                [0.0, 0.8, 0.0, 0.8],
                [0.0, 0.0, 0.8, 0.8],
            ];

            for (i, buffer) in self.buffers.iter().enumerate() {
                command_list.set_pipeline_state(&self.fill_pso);

                command_list.set_primivive_topology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);

                params.color = FILL_COLORS[i];
                command_list.set_graphics_constants(
                    RootParam::Constants,
                    NUM_CONSTS,
                    &params as *const _ as _,
                );

                command_list.set_graphics_descriptor_table(RootParam::Srvs, buffer.srv());

                command_list.draw(2 * 256, 1);

                command_list.set_pipeline_state(&self.line_pso);

                command_list.set_primivive_topology(D3D_PRIMITIVE_TOPOLOGY_LINESTRIP);

                params.color = LINE_COLORS[i];
                command_list.set_graphics_constants(
                    RootParam::Constants,
                    NUM_CONSTS,
                    &params as *const _ as _,
                );

                command_list.set_graphics_descriptor_table(RootParam::Srvs, buffer.srv());

                command_list.draw(256, 1);
            }
        } else {
            const FILL_COLOR: [f32; 4] = [0.8, 0.8, 0.8, 0.6];
            const LINE_COLOR: [f32; 4] = [0.8, 0.8, 0.8, 0.9];

            let buffer = &self.buffers[0];

            command_list.set_pipeline_state(&self.fill_pso);

            command_list.set_primivive_topology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);

            params.color = FILL_COLOR;
            command_list.set_graphics_constants(
                RootParam::Constants,
                NUM_CONSTS,
                &params as *const _ as _,
            );

            command_list.set_graphics_descriptor_table(RootParam::Srvs, buffer.srv());

            command_list.draw(2 * 256, 1);

            command_list.set_pipeline_state(&self.line_pso);

            command_list.set_primivive_topology(D3D_PRIMITIVE_TOPOLOGY_LINESTRIP);

            params.color = LINE_COLOR;
            command_list.set_graphics_constants(
                RootParam::Constants,
                NUM_CONSTS,
                &params as *const _ as _,
            );

            command_list.set_graphics_descriptor_table(RootParam::Srvs, buffer.srv());

            command_list.draw(256, 1);
        }

        Ok(())
    }
}
