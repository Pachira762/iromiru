use crate::graphics::{capture::Capture, context::*};
use crate::state::*;
use windows::core::*;
use windows::Win32::Graphics::Direct3D12::ID3D12RootSignature;

use super::color_cloud_count_pass::ColorCloudCountPass;
use super::color_cloud_indirect_pass::ColorCloudIndirectPass;
use super::color_cloud_mesh_pass::ColorCloudMeshPass;

pub struct ColorCloudPass {
    count_pass: ColorCloudCountPass,
    mesh_pass: Option<ColorCloudMeshPass>,
    indirect_pass: Option<ColorCloudIndirectPass>,
}

impl ColorCloudPass {
    pub fn new(context: &mut Context, root_signature: &ID3D12RootSignature) -> Result<Self> {
        let count_pass = ColorCloudCountPass::new(context, root_signature)?;

        let mesh_pass = ColorCloudMeshPass::new(context, root_signature).ok();

        let indirect_pass = match mesh_pass {
            Some(_) => None,
            None => Some(ColorCloudIndirectPass::new(context, root_signature)?),
        };

        Ok(Self {
            count_pass,
            mesh_pass,
            indirect_pass,
        })
    }

    pub fn process(
        &mut self,
        context: &mut Context,
        state: &State,
        capture: &Capture,
    ) -> Result<()> {
        if state.color_cloud_mode.is_enable() {
            self.count_pass.process(context, state, capture)?;

            if let Some(mesh_pass) = &mut self.mesh_pass {
                mesh_pass.process(context, state, capture)?;
            }

            if let Some(indirect_pass) = &mut self.indirect_pass {
                indirect_pass.process(context, state, capture)?;
            }
        }

        Ok(())
    }
}
