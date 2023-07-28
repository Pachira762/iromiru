use std::mem::size_of;

use windows::{
    core::*,
    w,
    Win32::{
        Foundation::RECT,
        Graphics::{Direct3D::*, Direct3D12::*},
    },
};

use crate::{
    graphics::{capture::Capture, context::*},
    state::{State, ViewMode},
};

use super::RootParam;

pub struct ViewPass {
    pso: ID3D12PipelineState,
}

impl ViewPass {
    pub fn new(context: &mut Context, root_signature: &ID3D12RootSignature) -> Result<Self> {
        let device = &context.device;
        let compiler = &context.compiler;

        let pso = device.create_graphics_pipeline(
            root_signature,
            &compiler.compile(w!("shaders\\view.hlsl"), w!("ViewVs"), w!("vs_6_0"), &[])?,
            &compiler.compile(w!("shaders\\view.hlsl"), w!("ViewPs"), w!("ps_6_0"), &[])?,
            BlendState::none(),
            RasterizerState::no_cull(),
            DepthStencilState::none(),
            &[],
            None,
            None,
            None,
            None,
        )?;

        Ok(Self { pso })
    }

    pub fn process(
        &mut self,
        context: &mut Context,
        state: &State,
        _capture: &Capture,
    ) -> Result<()> {
        if state.view_mode.is_enable() {
            self.view(context, state)?;
        }

        Ok(())
    }

    pub fn view(&self, context: &mut Context, state: &State) -> Result<()> {
        #[repr(C)]
        struct Params {
            rect: RECT,
            mask: [f32; 4],
            mode: u32,
        }
        const NUM_CONSTS: u32 = size_of::<Params>() as u32 / 4;

        let command_list = &context.command_list;

        command_list.set_pipeline_state(&self.pso);

        command_list.set_graphics_constants(
            RootParam::Constants,
            NUM_CONSTS,
            &Params {
                rect: state.rect,
                mask: get_mask(state),
                mode: get_mode(state),
            } as *const _ as _,
        );

        command_list.set_primivive_topology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);

        command_list.draw(6, 1);

        Ok(())
    }
}

fn get_mode(state: &State) -> u32 {
    match state.view_mode {
        ViewMode::Original => 0,
        ViewMode::Rgb(_) => 1,
        ViewMode::Hue => 2,
        ViewMode::Saturation => 3,
        ViewMode::Brightness => 4,
    }
}

fn get_mask(state: &State) -> [f32; 4] {
    let mut mask = [0.0; 4];

    if let ViewMode::Rgb(channel_mask) = state.view_mode {
        if channel_mask.at(0) {
            mask[0] = 1.0;
        }
        if channel_mask.at(1) {
            mask[1] = 1.0;
        }
        if channel_mask.at(2) {
            mask[2] = 1.0;
        }
    }

    mask
}
