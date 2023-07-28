use crate::graphics::{capture::Capture, context::*};
use crate::state::*;
use glam::*;
use std::mem::size_of;
use windows::core::*;
use windows::Win32::Graphics::Direct3D::Dxc::DxcDefine;
use windows::{w, Win32::Graphics::Direct3D12::*};

use super::RootParam;

pub struct ColorCloudMeshPass {
    draw_pso: ID3D12PipelineState,
}

impl ColorCloudMeshPass {
    pub fn new(context: &mut Context, root_signature: &ID3D12RootSignature) -> Result<Self> {
        let device = &context.device;
        let compiler = &context.compiler;

        let draw_pso = device.create_mesh_shader_pipeline(
            &root_signature,
            Some(&compiler.compile(
                w!("shaders\\color_cloud.hlsl"),
                w!("DrawAs"),
                w!("as_6_5"),
                &[
                    DxcDefine {
                        Name: w!("DRAW"),
                        Value: w!(""),
                    },
                    DxcDefine {
                        Name: w!("MESH"),
                        Value: w!(""),
                    },
                ],
            )?),
            &compiler.compile(
                w!("shaders\\color_cloud.hlsl"),
                w!("DrawMs"),
                w!("ms_6_5"),
                &[
                    DxcDefine {
                        Name: w!("DRAW"),
                        Value: w!(""),
                    },
                    DxcDefine {
                        Name: w!("MESH"),
                        Value: w!(""),
                    },
                ],
            )?,
            &compiler.compile(
                w!("shaders\\color_cloud.hlsl"),
                w!("DrawPs"),
                w!("ps_6_0"),
                &[
                    DxcDefine {
                        Name: w!("DRAW"),
                        Value: w!(""),
                    },
                    DxcDefine {
                        Name: w!("PIXEL"),
                        Value: w!(""),
                    },
                ],
            )?,
            BlendState::none(),
            RasterizerState::default(),
            DepthStencilState::depth(),
            D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
            None,
            None,
        )?;

        Ok(Self { draw_pso })
    }

    pub fn process(
        &mut self,
        context: &mut Context,
        state: &State,
        _capture: &Capture,
    ) -> Result<()> {
        if state.color_cloud_mode.is_enable() {
            self.draw(context, state)?;
        }
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

        command_list.set_graphics_constants(
            RootParam::Constants,
            size_of::<Params>() as u32 / 4,
            &params as *const _ as _,
        );

        const GRID: u32 = 8;
        command_list.dispatch_mesh(256 / GRID, 256 / GRID, 256 / GRID);

        Ok(())
    }
}

// use crate::graphics::{capture::Capture, context::*};
// use crate::state::*;
// use glam::*;
// use std::mem::size_of;
// use windows::core::*;
// use windows::Win32::Foundation::RECT;
// use windows::Win32::Graphics::Direct3D::Dxc::DxcDefine;
// use windows::Win32::Graphics::Dxgi::Common::*;
// use windows::{w, Win32::Graphics::Direct3D12::*};

// use super::RootParam;

// pub struct ColorCloudPass {
//     count_pso: ID3D12PipelineState,
//     draw_pso: ID3D12PipelineState,
//     count_buf: Resource,
// }

// impl ColorCloudPass {
//     pub fn new(context: &mut Context, root_signature: &ID3D12RootSignature) -> Result<Self> {
//         let device = &context.device;
//         let descriptor_heap = &mut context.descriptor_heap;
//         let compiler = &context.compiler;

//         let count_pso = device.create_compute_pipeline(
//             &root_signature,
//             &compiler.compile(
//                 w!("shaders\\color_cloud_mesh.hlsl"),
//                 w!("CountCs"),
//                 w!("cs_6_5"),
//                 &[
//                     DxcDefine {
//                         Name: w!("COUNT"),
//                         Value: w!(""),
//                     },
//                     DxcDefine {
//                         Name: w!("CS_6_5"),
//                         Value: w!(""),
//                     },
//                 ],
//             )?,
//         )?;

//         let draw_pso = device.create_mesh_shader_pipeline(
//             &root_signature,
//             Some(&compiler.compile(
//                 w!("shaders\\color_cloud_mesh.hlsl"),
//                 w!("DrawAs"),
//                 w!("as_6_5"),
//                 &[
//                     DxcDefine {
//                         Name: w!("DRAW"),
//                         Value: w!(""),
//                     },
//                     DxcDefine {
//                         Name: w!("MESH"),
//                         Value: w!(""),
//                     },
//                 ],
//             )?),
//             &compiler.compile(
//                 w!("shaders\\color_cloud_mesh.hlsl"),
//                 w!("DrawMs"),
//                 w!("ms_6_5"),
//                 &[
//                     DxcDefine {
//                         Name: w!("DRAW"),
//                         Value: w!(""),
//                     },
//                     DxcDefine {
//                         Name: w!("MESH"),
//                         Value: w!(""),
//                     },
//                 ],
//             )?,
//             &compiler.compile(
//                 w!("shaders\\color_cloud_mesh.hlsl"),
//                 w!("DrawPs"),
//                 w!("ps_6_0"),
//                 &[
//                     DxcDefine {
//                         Name: w!("DRAW"),
//                         Value: w!(""),
//                     },
//                     DxcDefine {
//                         Name: w!("PIXEL"),
//                         Value: w!(""),
//                     },
//                 ],
//             )?,
//             BlendState::none(),
//             RasterizerState::default(),
//             DepthStencilState::depth(),
//             D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
//             None,
//             None,
//         )?;

//         let mut count_buf = Resource::new_buffer(
//             &device,
//             4 * 256 * 256 * 256,
//             D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
//             D3D12_RESOURCE_STATE_COMMON,
//         )?;

//         descriptor_heap.create_srv_buffer(
//             &mut count_buf,
//             Some(DXGI_FORMAT_R32_UINT),
//             None,
//             256 * 256 * 256,
//         );

//         descriptor_heap.create_uav_buffer(
//             &mut count_buf,
//             Some(DXGI_FORMAT_R32_UINT),
//             None,
//             256 * 256 * 256,
//             None,
//             None,
//         );

//         descriptor_heap.create_uav_to_clear(&mut count_buf, 256 * 256 * 256, 0);

//         Ok(Self {
//             count_pso,
//             draw_pso,
//             count_buf,
//         })
//     }

//     pub fn process(
//         &mut self,
//         context: &mut Context,
//         state: &State,
//         _capture: &Capture,
//     ) -> Result<()> {
//         if state.color_cloud_mode.is_enable() {
//             self.clear(context)?;
//             self.count(context, state)?;
//             self.draw(context, state)?;
//         }
//         Ok(())
//     }

//     #[allow(unused)]
//     pub fn dump(&self) -> Result<()> {
//         unsafe { Ok(()) }
//     }

//     fn clear(&mut self, context: &mut Context) -> Result<()> {
//         let command_list = &context.command_list;

//         command_list.resource_barrier(&[ResourceBarrier::transition(
//             &self.count_buf,
//             D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
//             D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
//         )]);

//         command_list.clear_unordered_access_view_uint(&self.count_buf, &[0, 0, 0, 0], &[]);

//         Ok(())
//     }

//     fn count(&mut self, context: &mut Context, state: &State) -> Result<()> {
//         #[repr(C)]
//         struct Params {
//             rect: RECT,
//         }

//         let rect = state.rect;
//         let width = rect_width(&rect) as u32;
//         let height = rect_height(&rect) as u32;

//         const THREAD: u32 = 8;
//         let dim_x = div_round_up(width, THREAD);
//         let dim_y = div_round_up(height, THREAD);

//         let command_list = &context.command_list;
//         command_list.set_pipeline_state(&self.count_pso);

//         command_list.set_compute_constants(
//             RootParam::Constants,
//             size_of::<Params>() as u32 / 4,
//             &Params { rect: state.rect } as *const _ as _,
//         );

//         command_list.set_compute_descriptor_table(RootParam::Uavs, self.count_buf.uav());

//         command_list.dispatch(dim_x, dim_y, 1);

//         Ok(())
//     }

//     fn draw(&self, context: &mut Context, state: &State) -> Result<()> {
//         #[repr(C)]
//         struct Params {
//             projection: Mat4,
//             scale: Vec2,
//             num_pixels: u32,
//             color_space: u32,
//         }

//         let (width, height) = rect_size(&state.rect);
//         let aspect = width as f32 / height as f32;

//         let params = Params {
//             projection: Mat4::from_quat(state.rotation).inverse(),
//             scale: if aspect > 1.0 {
//                 Vec2::new(1.0 / aspect, 1.0)
//             } else {
//                 Vec2::new(1.0, 1.0 * aspect)
//             },
//             num_pixels: (width * height) as _,
//             color_space: state.color_cloud_mode.color_space().unwrap() as _,
//         };

//         let command_list = &context.command_list;

//         command_list.set_pipeline_state(&self.draw_pso);

//         command_list.resource_barrier(&[ResourceBarrier::transition(
//             &self.count_buf,
//             D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
//             D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
//         )]);

//         command_list.set_graphics_descriptor_table(RootParam::Srvs, self.count_buf.srv());

//         command_list.set_graphics_constants(
//             RootParam::Constants,
//             size_of::<Params>() as u32 / 4,
//             &params as *const _ as _,
//         );

//         const GRID: u32 = 8;
//         command_list.dispatch_mesh(256 / GRID, 256 / GRID, 256 / GRID);

//         Ok(())
//     }
// }
