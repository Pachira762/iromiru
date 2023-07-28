use windows::core::*;
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT;
use windows::Win32::{
    Foundation::*,
    Graphics::{Direct3D::*, Direct3D12::*},
    System::Threading::*,
};

use super::context::Resource;
use super::descriptor::{Descriptor, DescriptorHeap};
use super::device::Device;

pub struct CommandList {
    command_queue: ID3D12CommandQueue,
    command_allocator: ID3D12CommandAllocator,
    command_list: ID3D12GraphicsCommandList6,

    fence: ID3D12Fence,
    fence_value: u64,
    fence_event: HANDLE,
}

impl CommandList {
    pub fn new(device: &Device) -> Result<Self> {
        unsafe {
            let device = &device.device;

            let command_queue: ID3D12CommandQueue =
                device.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
                    Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                    ..Default::default()
                })?;

            let command_allocator =
                device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)?;

            let command_list: ID3D12GraphicsCommandList6 = device.CreateCommandList(
                0,
                D3D12_COMMAND_LIST_TYPE_DIRECT,
                &command_allocator,
                None,
            )?;
            command_list.Close()?;

            let fence = device.CreateFence(0, D3D12_FENCE_FLAG_NONE)?;
            let fence_value = 1;
            let fence_event = CreateEventA(None, FALSE, FALSE, None)?;

            Ok(Self {
                command_queue,
                command_allocator,
                command_list,
                fence,
                fence_value,
                fence_event,
            })
        }
    }

    pub fn reset(&self) -> Result<()> {
        unsafe {
            self.command_allocator.Reset()?;
            self.command_list.Reset(&self.command_allocator, None)?;
        }
        Ok(())
    }

    pub fn execute(&mut self) -> Result<()> {
        unsafe {
            self.command_list.Close()?;

            self.command_queue
                .ExecuteCommandLists(&[Some(self.command_list.cast()?)]);
        }
        Ok(())
    }

    pub fn set_descriptor_heap(&self, descriptor_heap: &DescriptorHeap) {
        unsafe {
            self.command_list
                .SetDescriptorHeaps(&[Some(descriptor_heap.heap().clone())]);
        }
    }

    #[allow(unused)]
    pub fn set_pipeline_state(&self, pso: &ID3D12PipelineState) {
        unsafe {
            self.command_list.SetPipelineState(pso);
        }
    }

    #[allow(unused)]
    pub fn set_compute_root_signature(&self, root_signature: &ID3D12RootSignature) {
        unsafe {
            self.command_list.SetComputeRootSignature(root_signature);
        }
    }

    #[allow(unused)]
    pub fn set_graphics_root_signature(&self, root_signature: &ID3D12RootSignature) {
        unsafe {
            self.command_list.SetGraphicsRootSignature(root_signature);
        }
    }

    #[allow(unused)]
    pub fn set_compute_descriptor_table<T: Into<u32>>(
        &self,
        root_index: T,
        base_descriptor: &Descriptor,
    ) {
        unsafe {
            self.command_list
                .SetComputeRootDescriptorTable(root_index.into(), base_descriptor.gpu);
        }
    }

    #[allow(unused)]
    pub fn set_graphics_descriptor_table<T: Into<u32>>(
        &self,
        root_index: T,
        base_descriptor: &Descriptor,
    ) {
        unsafe {
            self.command_list
                .SetGraphicsRootDescriptorTable(root_index.into(), base_descriptor.gpu);
        }
    }

    #[allow(unused)]
    pub fn set_compute_constants<T: Into<u32>>(
        &self,
        root_index: T,
        num_constants: u32,
        constants: *const u8,
    ) {
        unsafe {
            self.command_list.SetComputeRoot32BitConstants(
                root_index.into(),
                num_constants,
                constants as _,
                0,
            );
        }
    }

    #[allow(unused)]
    pub fn set_graphics_constants<T: Into<u32>>(
        &self,
        root_index: T,
        num_constants: u32,
        constants: *const u8,
    ) {
        unsafe {
            self.command_list.SetGraphicsRoot32BitConstants(
                root_index.into(),
                num_constants,
                constants as _,
                0,
            );
        }
    }

    pub fn resource_barrier(&self, barriers: &[D3D12_RESOURCE_BARRIER]) {
        unsafe {
            self.command_list.ResourceBarrier(barriers);
        }
    }

    pub fn clear_unordered_access_view_uint(
        &self,
        resource: &Resource,
        values: &[u32; 4],
        rects: &[RECT],
    ) {
        unsafe {
            let (shader_visible, non_shader_visible) = resource.uav_to_clear();

            self.command_list.ClearUnorderedAccessViewUint(
                shader_visible.gpu,
                non_shader_visible.cpu,
                &resource.resource,
                values.as_ptr(),
                rects,
            );
        }
    }

    pub fn clear_render_target_view(&self, descriptor: &Descriptor, color: &[f32; 4]) {
        unsafe {
            self.command_list
                .ClearRenderTargetView(descriptor.cpu, color.as_ptr(), None);
        }
    }

    pub fn clear_depth_stencil_view(&self, descriptor: &Descriptor) {
        unsafe {
            self.command_list.ClearDepthStencilView(
                descriptor.cpu,
                D3D12_CLEAR_FLAG_DEPTH,
                1.0,
                0,
                &[],
            );
        }
    }

    pub fn set_render_target(&self, rtv: &Descriptor, dsv: &Descriptor) {
        unsafe {
            self.command_list
                .OMSetRenderTargets(1, Some(&rtv.cpu), false, Some(&dsv.cpu));
        }
    }

    pub fn set_viewport(&self, x: f32, y: f32, width: f32, height: f32) {
        unsafe {
            self.command_list.RSSetViewports(&[D3D12_VIEWPORT {
                TopLeftX: x,
                TopLeftY: y,
                Width: width,
                Height: height,
                MinDepth: 0.0,
                MaxDepth: 1.0,
            }]);
        }
    }

    pub fn set_scissor_rect(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe {
            self.command_list.RSSetScissorRects(&[RECT {
                left: x,
                top: y,
                right: x + width,
                bottom: y + height,
            }]);
        }
    }

    pub fn set_primivive_topology(&self, primitive_topology: D3D_PRIMITIVE_TOPOLOGY) {
        unsafe {
            self.command_list.IASetPrimitiveTopology(primitive_topology);
        }
    }

    #[allow(unused)]
    pub fn set_vertex_buffer(&self, slot: u32, views: &[D3D12_VERTEX_BUFFER_VIEW]) {
        unsafe {
            self.command_list.IASetVertexBuffers(slot, Some(views));
        }
    }

    pub fn dispatch(&self, num_groups_x: u32, num_groups_y: u32, num_groups_z: u32) {
        unsafe {
            self.command_list
                .Dispatch(num_groups_x, num_groups_y, num_groups_z);
        }
    }

    pub fn draw(&self, num_vertices: u32, num_instances: u32) {
        unsafe {
            self.command_list
                .DrawInstanced(num_vertices, num_instances, 0, 0);
        }
    }

    #[allow(unused)]
    pub fn execute_indirect(
        &self,
        command_signature: &ID3D12CommandSignature,
        max_commands: u32,
        arg_buf: &ID3D12Resource,
        arg_buf_offset: u64,
        count_buf: Option<&ID3D12Resource>,
        count_buf_offset: u64,
    ) {
        unsafe {
            self.command_list.ExecuteIndirect(
                command_signature,
                max_commands,
                arg_buf,
                arg_buf_offset,
                count_buf,
                count_buf_offset,
            );
        }
    }

    pub fn dispatch_mesh(&self, num_groups_x: u32, num_groups_y: u32, num_groups_z: u32) {
        unsafe {
            self.command_list
                .DispatchMesh(num_groups_x, num_groups_y, num_groups_z);
        }
    }

    #[allow(unused)]
    pub fn resolve_resource(
        &self,
        dest: &ID3D12Resource,
        src: &ID3D12Resource,
        format: DXGI_FORMAT,
    ) {
        unsafe {
            self.command_list
                .ResolveSubresource(dest, 0, src, 0, format);
        }
    }

    pub fn end_query(
        &self,
        query_heap: &ID3D12QueryHeap,
        query_type: D3D12_QUERY_TYPE,
        index: u32,
    ) {
        unsafe {
            self.command_list.EndQuery(query_heap, query_type, index);
        }
    }

    pub fn resolve_query_data(
        &self,
        query_heap: &ID3D12QueryHeap,
        query_type: D3D12_QUERY_TYPE,
        num: u32,
        dest: &ID3D12Resource,
    ) {
        unsafe {
            self.command_list
                .ResolveQueryData(query_heap, query_type, 0, num, dest, 0);
        }
    }

    pub fn wait(&mut self) -> Result<()> {
        unsafe {
            let fence = self.fence_value;
            self.command_queue.Signal(&self.fence, fence)?;
            self.fence_value += 1;

            if self.fence.GetCompletedValue() < fence {
                self.fence.SetEventOnCompletion(fence, self.fence_event)?;
                WaitForSingleObject(self.fence_event, INFINITE).ok()?;
            }
        }
        Ok(())
    }

    pub fn queue(&self) -> &ID3D12CommandQueue {
        &self.command_queue
    }
}

pub struct ResourceBarrier();

impl ResourceBarrier {
    pub fn transition(
        resource: &ID3D12Resource,
        before: D3D12_RESOURCE_STATES,
        after: D3D12_RESOURCE_STATES,
    ) -> D3D12_RESOURCE_BARRIER {
        D3D12_RESOURCE_BARRIER {
            Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
            Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
            Anonymous: D3D12_RESOURCE_BARRIER_0 {
                Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                    pResource: unsafe { std::mem::transmute_copy(resource) },
                    StateBefore: before,
                    StateAfter: after,
                    Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                }),
            },
        }
    }
}

pub fn div_round_up(n: u32, d: u32) -> u32 {
    (n + d - 1) / d
}
