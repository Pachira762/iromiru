use crate::graphics::{capture::Capture, context::*};
use crate::state::*;
use glam::*;
use std::mem::size_of;
use windows::core::*;
use windows::Win32::Graphics::Direct3D::Dxc::DxcDefine;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::{w, Win32::Graphics::Direct3D12::*};

use super::RootParam;

#[repr(C)]
struct IndirectCommand {
    args: D3D12_DRAW_ARGUMENTS,
}

const COMPACT_BLOCK: u32 = 4;
const COMPACT_THREAD: u32 = 8;
const GRID: u32 = 256 / (COMPACT_BLOCK * COMPACT_THREAD);
const GRID3: u32 = GRID * GRID * GRID;

const COMMAND_SIZE: u32 = std::mem::size_of::<IndirectCommand>() as _;
const COUNT_OFFSET: u32 = 4096 * (((COMMAND_SIZE * GRID3) + 4096 - 1) / 4096);

pub struct ColorCloudIndirectPass {
    command_signature: ID3D12CommandSignature,
    compact_pso: ID3D12PipelineState,
    draw_pso: ID3D12PipelineState,

    packed_buf: Resource,
    command_buf: Resource,
}

impl ColorCloudIndirectPass {
    pub fn new(context: &mut Context, root_signature: &ID3D12RootSignature) -> Result<Self> {
        let device = &context.device;
        let descriptor_heap = &mut context.descriptor_heap;
        let compiler = &context.compiler;

        let command_signature = context.device.create_command_signature(
            COMMAND_SIZE,
            &[D3D12_INDIRECT_ARGUMENT_DESC {
                Type: D3D12_INDIRECT_ARGUMENT_TYPE_DRAW,
                ..Default::default()
            }],
            None,
        )?;

        let compact_pso = device.create_compute_pipeline(
            &root_signature,
            &compiler.compile(
                w!("shaders\\color_cloud.hlsl"),
                w!("CompactCs"),
                w!("cs_6_5"),
                &[DxcDefine {
                    Name: w!("COMPACT"),
                    Value: w!(""),
                }],
            )?,
        )?;

        let draw_pso = device.create_graphics_pipeline(
            &root_signature,
            &compiler.compile(
                w!("shaders\\color_cloud.hlsl"),
                w!("DrawVs"),
                w!("vs_6_0"),
                &[
                    DxcDefine {
                        Name: w!("DRAW"),
                        Value: w!(""),
                    },
                    DxcDefine {
                        Name: w!("INDIRECT"),
                        Value: w!(""),
                    },
                ],
            )?,
            &compiler.compile(
                w!("shaders\\color_cloud.hlsl"),
                w!("DrawPs"),
                w!("ps_6_0"),
                &[DxcDefine {
                    Name: w!("DRAW"),
                    Value: w!(""),
                }],
            )?,
            BlendState::none(),
            RasterizerState::default(),
            DepthStencilState::depth(),
            &[InputElement::per_instance(
                s!("COLOR_INDEX"),
                DXGI_FORMAT_R32_UINT,
                0,
            )],
            None,
            None,
            None,
            None,
        )?;

        let mut packed_buf = Resource::new_buffer(
            &device,
            4 * 256 * 256 * 256,
            D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE | D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            D3D12_RESOURCE_STATE_COMMON,
        )?;

        let mut command_buf = Resource::new_buffer(
            &device,
            (COUNT_OFFSET + 4) as _,
            D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            D3D12_RESOURCE_STATE_COMMON,
        )?;

        descriptor_heap.create_uav_buffer(
            &mut packed_buf,
            Some(DXGI_FORMAT_R32_UINT),
            None,
            256 * 256 * 256,
            None,
            None,
        );

        descriptor_heap.create_uav_buffer(
            &mut command_buf,
            None,
            Some(COMMAND_SIZE),
            GRID3,
            None,
            Some(COUNT_OFFSET as _),
        );

        descriptor_heap.create_uav_to_clear(&mut command_buf, 1, (COUNT_OFFSET / 4) as _);

        Ok(Self {
            command_signature,
            compact_pso,
            draw_pso,
            packed_buf,
            command_buf,
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
            self.pack(context)?;
            self.draw(context, state)?;
        }
        Ok(())
    }

    fn clear(&mut self, context: &mut Context) -> Result<()> {
        let command_list = &context.command_list;

        command_list.resource_barrier(&[ResourceBarrier::transition(
            &self.command_buf,
            D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
            D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
        )]);

        command_list.clear_unordered_access_view_uint(&self.command_buf, &[0, 0, 0, 0], &[]);

        Ok(())
    }

    fn pack(&self, context: &mut Context) -> Result<()> {
        let command_list = &context.command_list;
        command_list.set_pipeline_state(&self.compact_pso);

        command_list.set_compute_descriptor_table(RootParam::Uavs, self.packed_buf.uav());

        command_list.dispatch(GRID, GRID, GRID);

        Ok(())
    }

    fn draw(&self, context: &mut Context, state: &State) -> Result<()> {
        #[repr(C)]
        struct Params {
            projection: Mat4,
            scale: Vec2,
            num_pixels: u32,
            color_space: u32,
        }

        let (width, height) = rect_size(&state.rect);
        let aspect = width as f32 / height as f32;

        let params = Params {
            projection: Mat4::from_quat(state.rotation).inverse(),
            scale: if aspect > 1.0 {
                Vec2::new(1.0 / aspect, 1.0)
            } else {
                Vec2::new(1.0, 1.0 * aspect)
            },
            num_pixels: (width * height) as _,
            color_space: state.color_cloud_mode.color_space().unwrap() as _,
        };

        let command_list = &context.command_list;
        command_list.set_pipeline_state(&self.draw_pso);

        command_list.resource_barrier(&[ResourceBarrier::transition(
            &self.command_buf,
            D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
        )]);

        command_list.set_graphics_constants(
            RootParam::Constants,
            size_of::<Params>() as u32 / 4,
            &params as *const _ as _,
        );

        command_list.set_primivive_topology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);

        command_list.set_vertex_buffer(
            0,
            &[D3D12_VERTEX_BUFFER_VIEW {
                BufferLocation: unsafe { self.packed_buf.GetGPUVirtualAddress() },
                SizeInBytes: 4 * 256 * 256 * 256,
                StrideInBytes: 4,
            }],
        );

        command_list.execute_indirect(
            &self.command_signature,
            GRID3,
            &self.command_buf,
            0,
            Some(&self.command_buf),
            COUNT_OFFSET as _,
        );

        Ok(())
    }
}
