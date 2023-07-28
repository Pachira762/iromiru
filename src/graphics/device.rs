use std::mem::size_of;
use std::mem::transmute_copy;

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::*;

use super::blob::Blob;
use super::context::SampleDesc;

#[derive(Clone)]
pub struct Device {
    pub device: ID3D12Device5,

    #[allow(unused)]
    debug: Option<ID3D12Debug>,
}

impl Device {
    pub fn new() -> Result<Self> {
        unsafe {
            let debug = if cfg!(debug_assertions) {
                let mut debug: Option<ID3D12Debug> = None;
                D3D12GetDebugInterface(&mut debug)?;

                let debug = debug.unwrap();
                debug.EnableDebugLayer();
                Some(debug)
            } else {
                None
            };

            let mut device = None;
            D3D12CreateDevice(None, D3D_FEATURE_LEVEL_11_0, &mut device)?;
            let device: ID3D12Device5 = device.unwrap();

            Ok(Self { debug, device })
        }
    }

    pub fn create_resource(
        &self,
        props: &D3D12_HEAP_PROPERTIES,
        heap_flags: D3D12_HEAP_FLAGS,
        desc: &D3D12_RESOURCE_DESC,
        initial_state: D3D12_RESOURCE_STATES,
        clear_value: Option<*const D3D12_CLEAR_VALUE>,
    ) -> Result<ID3D12Resource> {
        unsafe {
            let mut resource = None;
            self.device.CreateCommittedResource(
                props,
                heap_flags,
                desc,
                initial_state,
                clear_value,
                &mut resource,
            )?;
            Ok(resource.unwrap())
        }
    }

    pub fn open_shared_handle(&self, handle: HANDLE) -> Result<ID3D12Resource> {
        unsafe {
            let mut resource = None;
            self.device.OpenSharedHandle(handle, &mut resource)?;
            Ok(resource.unwrap())
        }
    }

    pub fn create_descriptor_heap(
        &self,
        heap_type: D3D12_DESCRIPTOR_HEAP_TYPE,
        num: u32,
        flags: D3D12_DESCRIPTOR_HEAP_FLAGS,
    ) -> Result<ID3D12DescriptorHeap> {
        unsafe {
            self.device
                .CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                    Type: heap_type,
                    NumDescriptors: num,
                    Flags: flags,
                    NodeMask: 0,
                })
        }
    }

    pub fn create_shader_resource_view(
        &self,
        resource: &ID3D12Resource,
        desc: Option<*const D3D12_SHADER_RESOURCE_VIEW_DESC>,
        descriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
    ) {
        unsafe {
            self.device
                .CreateShaderResourceView(resource, desc, descriptor)
        }
    }

    pub fn create_unordered_access_view(
        &self,
        resource: &ID3D12Resource,
        counter: Option<&ID3D12Resource>,
        desc: Option<*const D3D12_UNORDERED_ACCESS_VIEW_DESC>,
        descriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
    ) {
        unsafe {
            self.device
                .CreateUnorderedAccessView(resource, counter, desc, descriptor)
        }
    }

    pub fn create_render_target_view(
        &self,
        resource: &ID3D12Resource,
        desc: Option<*const D3D12_RENDER_TARGET_VIEW_DESC>,
        descriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
    ) {
        unsafe {
            self.device
                .CreateRenderTargetView(resource, desc, descriptor)
        }
    }

    pub fn create_depth_stencil_view(
        &self,
        resource: &ID3D12Resource,
        desc: Option<*const D3D12_DEPTH_STENCIL_VIEW_DESC>,
        descriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
    ) {
        unsafe {
            self.device
                .CreateDepthStencilView(resource, desc, descriptor)
        }
    }

    pub fn descriptor_size(&self, heap_type: D3D12_DESCRIPTOR_HEAP_TYPE) -> u32 {
        unsafe { self.device.GetDescriptorHandleIncrementSize(heap_type) }
    }

    pub fn create_root_signature(
        &self,
        params: &[D3D12_ROOT_PARAMETER],
        sampler: &[D3D12_STATIC_SAMPLER_DESC],
        flags: D3D12_ROOT_SIGNATURE_FLAGS,
    ) -> Result<ID3D12RootSignature> {
        unsafe {
            let mut blob = Blob::D3D(None);
            let mut error = Blob::D3D(None);
            let ret = D3D12SerializeRootSignature(
                &D3D12_ROOT_SIGNATURE_DESC {
                    NumParameters: params.len() as _,
                    pParameters: params.as_ptr(),
                    NumStaticSamplers: sampler.len() as _,
                    pStaticSamplers: sampler.as_ptr(),
                    Flags: flags,
                },
                D3D_ROOT_SIGNATURE_VERSION_1,
                blob.d3d_option(),
                Some(error.d3d_option()),
            );

            if !error.is_empty() {
                println!("{:?}", error.as_str());
            }
            ret?;

            self.device.CreateRootSignature(0, blob.as_slice())
        }
    }

    #[allow(unused)]
    pub fn create_command_signature(
        &self,
        stride: u32,
        descs: &[D3D12_INDIRECT_ARGUMENT_DESC],
        root_signature: Option<&ID3D12RootSignature>,
    ) -> Result<ID3D12CommandSignature> {
        unsafe {
            let mut command_signature = None;

            self.device.CreateCommandSignature(
                &D3D12_COMMAND_SIGNATURE_DESC {
                    ByteStride: stride,
                    NumArgumentDescs: descs.len() as _,
                    pArgumentDescs: descs.as_ptr(),
                    NodeMask: 0,
                },
                root_signature,
                &mut command_signature,
            )?;

            Ok(command_signature.unwrap())
        }
    }

    pub fn create_graphics_pipeline(
        &self,
        root_signature: &ID3D12RootSignature,
        vs: &Blob,
        ps: &Blob,
        blend: D3D12_BLEND_DESC,
        rasterizer: D3D12_RASTERIZER_DESC,
        depth_stencil: D3D12_DEPTH_STENCIL_DESC,
        input_elements: &[D3D12_INPUT_ELEMENT_DESC],
        primitive_topology: Option<D3D12_PRIMITIVE_TOPOLOGY_TYPE>,
        rtv_format: Option<DXGI_FORMAT>,
        dsv_format: Option<DXGI_FORMAT>,
        sample_desc: Option<DXGI_SAMPLE_DESC>,
    ) -> Result<ID3D12PipelineState> {
        unsafe {
            self.device
                .CreateGraphicsPipelineState(&D3D12_GRAPHICS_PIPELINE_STATE_DESC {
                    pRootSignature: std::mem::transmute_copy(root_signature),
                    VS: vs.as_bytecode(),
                    PS: ps.as_bytecode(),
                    BlendState: blend,
                    SampleMask: D3D12_DEFAULT_SAMPLE_MASK,
                    RasterizerState: rasterizer,
                    DepthStencilState: depth_stencil,
                    InputLayout: D3D12_INPUT_LAYOUT_DESC {
                        pInputElementDescs: input_elements.as_ptr(),
                        NumElements: input_elements.len() as _,
                    },
                    PrimitiveTopologyType: primitive_topology
                        .unwrap_or(D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE),
                    NumRenderTargets: 1,
                    RTVFormats: RtvFormats::single(rtv_format),
                    DSVFormat: dsv_format.unwrap_or(DXGI_FORMAT_D32_FLOAT),
                    SampleDesc: sample_desc.unwrap_or(SampleDesc::default()),
                    ..Default::default()
                })
        }
    }

    pub fn create_compute_pipeline(
        &self,
        root_signature: &ID3D12RootSignature,
        cs: &Blob,
    ) -> Result<ID3D12PipelineState> {
        unsafe {
            self.device
                .CreateComputePipelineState(&D3D12_COMPUTE_PIPELINE_STATE_DESC {
                    pRootSignature: std::mem::transmute_copy(root_signature),
                    CS: cs.as_bytecode(),
                    NodeMask: 0,
                    CachedPSO: Default::default(),
                    Flags: D3D12_PIPELINE_STATE_FLAG_NONE,
                })
        }
    }

    pub fn create_mesh_shader_pipeline(
        &self,
        root_signature: &ID3D12RootSignature,
        r#as: Option<&Blob>,
        ms: &Blob,
        ps: &Blob,
        blend: D3D12_BLEND_DESC,
        rasterizer: D3D12_RASTERIZER_DESC,
        depth_stencil: D3D12_DEPTH_STENCIL_DESC,
        primitive_topology: D3D12_PRIMITIVE_TOPOLOGY_TYPE,
        rtv_format: Option<DXGI_FORMAT>,
        dsv_format: Option<DXGI_FORMAT>,
    ) -> Result<ID3D12PipelineState> {
        unsafe {
            let state_stream = MeshShaderPipelineStateDesc {
                root_signature: PipelineSubject::new(transmute_copy(root_signature)),
                r#as: PipelineSubject::new(
                    r#as.and_then(|blob| Some(blob.as_bytecode()))
                        .unwrap_or_default(),
                ),
                ms: PipelineSubject::new(ms.as_bytecode()),
                ps: PipelineSubject::new(ps.as_bytecode()),
                blend: PipelineSubject::new(blend),
                rasterizer: PipelineSubject::new(rasterizer),
                sample_mask: PipelineSubject::new(D3D12_DEFAULT_SAMPLE_MASK),
                depth_stencil: PipelineSubject::new(depth_stencil),
                primitive_topology: PipelineSubject::new(primitive_topology),
                rtv_formats: PipelineSubject::new(D3D12_RT_FORMAT_ARRAY {
                    RTFormats: RtvFormats::single(rtv_format),
                    NumRenderTargets: 1,
                }),
                dsv_format: PipelineSubject::new(dsv_format.unwrap_or(DXGI_FORMAT_D32_FLOAT)),
                sample_desc: PipelineSubject::new(SampleDesc::default()),
                ..Default::default()
            };

            self.device
                .CreatePipelineState(&D3D12_PIPELINE_STATE_STREAM_DESC {
                    SizeInBytes: size_of::<MeshShaderPipelineStateDesc>(),
                    pPipelineStateSubobjectStream: &state_stream as *const _ as _,
                })
        }
    }

    pub fn create_query_heap(
        &self,
        heap_type: D3D12_QUERY_HEAP_TYPE,
        num: u32,
    ) -> Result<ID3D12QueryHeap> {
        unsafe {
            let mut heap = None;
            self.device.CreateQueryHeap(
                &D3D12_QUERY_HEAP_DESC {
                    Type: heap_type,
                    Count: num,
                    NodeMask: 0,
                },
                &mut heap,
            )?;
            Ok(heap.unwrap())
        }
    }
}

pub struct BlendState();

impl BlendState {
    pub fn none() -> D3D12_BLEND_DESC {
        D3D12_BLEND_DESC {
            AlphaToCoverageEnable: FALSE,
            IndependentBlendEnable: FALSE,
            RenderTarget: [
                D3D12_RENDER_TARGET_BLEND_DESC {
                    BlendEnable: FALSE,
                    LogicOpEnable: FALSE,
                    SrcBlend: D3D12_BLEND_ONE,
                    DestBlend: D3D12_BLEND_ZERO,
                    BlendOp: D3D12_BLEND_OP_ADD,
                    SrcBlendAlpha: D3D12_BLEND_ONE,
                    DestBlendAlpha: D3D12_BLEND_ZERO,
                    BlendOpAlpha: D3D12_BLEND_OP_ADD,
                    LogicOp: D3D12_LOGIC_OP_NOOP,
                    RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as _,
                },
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ],
        }
    }

    #[allow(unused)]
    pub fn alpha() -> D3D12_BLEND_DESC {
        D3D12_BLEND_DESC {
            AlphaToCoverageEnable: FALSE,
            IndependentBlendEnable: FALSE,
            RenderTarget: [
                D3D12_RENDER_TARGET_BLEND_DESC {
                    BlendEnable: TRUE,
                    LogicOpEnable: FALSE,
                    SrcBlend: D3D12_BLEND_SRC_ALPHA,
                    DestBlend: D3D12_BLEND_INV_SRC_ALPHA,
                    BlendOp: D3D12_BLEND_OP_ADD,
                    SrcBlendAlpha: D3D12_BLEND_ONE,
                    DestBlendAlpha: D3D12_BLEND_ZERO,
                    BlendOpAlpha: D3D12_BLEND_OP_ADD,
                    LogicOp: D3D12_LOGIC_OP_NOOP,
                    RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as _,
                },
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ],
        }
    }
}

pub struct RasterizerState();

impl RasterizerState {
    pub fn default() -> D3D12_RASTERIZER_DESC {
        D3D12_RASTERIZER_DESC {
            FillMode: D3D12_FILL_MODE_SOLID,
            CullMode: D3D12_CULL_MODE_BACK,
            DepthClipEnable: TRUE,
            ..Default::default()
        }
    }

    pub fn no_cull() -> D3D12_RASTERIZER_DESC {
        D3D12_RASTERIZER_DESC {
            FillMode: D3D12_FILL_MODE_SOLID,
            CullMode: D3D12_CULL_MODE_NONE,
            DepthClipEnable: FALSE,
            // ConservativeRaster: D3D12_CONSERVATIVE_RASTERIZATION_MODE_ON,
            ..Default::default()
        }
    }
}

pub struct DepthStencilState();

impl DepthStencilState {
    pub fn none() -> D3D12_DEPTH_STENCIL_DESC {
        D3D12_DEPTH_STENCIL_DESC {
            DepthEnable: FALSE,
            StencilEnable: FALSE,
            ..Default::default()
        }
    }

    pub fn depth() -> D3D12_DEPTH_STENCIL_DESC {
        D3D12_DEPTH_STENCIL_DESC {
            DepthEnable: TRUE,
            DepthWriteMask: D3D12_DEPTH_WRITE_MASK_ALL,
            DepthFunc: D3D12_COMPARISON_FUNC_LESS,
            StencilEnable: FALSE,
            ..Default::default()
        }
    }
}

pub struct InputElement();

impl InputElement {
    #[allow(unused)]
    pub fn per_vertex(name: PCSTR, format: DXGI_FORMAT, slot: u32) -> D3D12_INPUT_ELEMENT_DESC {
        D3D12_INPUT_ELEMENT_DESC {
            SemanticName: name,
            SemanticIndex: 0,
            Format: format,
            InputSlot: slot,
            AlignedByteOffset: D3D12_APPEND_ALIGNED_ELEMENT,
            InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        }
    }

    #[allow(unused)]
    pub fn per_instance(name: PCSTR, format: DXGI_FORMAT, slot: u32) -> D3D12_INPUT_ELEMENT_DESC {
        D3D12_INPUT_ELEMENT_DESC {
            SemanticName: name,
            SemanticIndex: 0,
            Format: format,
            InputSlot: slot,
            AlignedByteOffset: D3D12_APPEND_ALIGNED_ELEMENT,
            InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_INSTANCE_DATA,
            InstanceDataStepRate: 1,
        }
    }
}

pub struct RtvFormats();

impl RtvFormats {
    fn single(format: Option<DXGI_FORMAT>) -> [DXGI_FORMAT; 8] {
        [
            format.unwrap_or(DXGI_FORMAT_R8G8B8A8_UNORM),
            DXGI_FORMAT_UNKNOWN,
            DXGI_FORMAT_UNKNOWN,
            DXGI_FORMAT_UNKNOWN,
            DXGI_FORMAT_UNKNOWN,
            DXGI_FORMAT_UNKNOWN,
            DXGI_FORMAT_UNKNOWN,
            DXGI_FORMAT_UNKNOWN,
        ]
    }
}

#[repr(C, align(8))]
struct PipelineSubject<const T: i32, Inner: Default> {
    subject_type: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    inner: Inner,
}

impl<const T: i32, Inner: Default> PipelineSubject<T, Inner> {
    pub fn new(inner: Inner) -> Self {
        Self {
            subject_type: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE(T),
            inner,
        }
    }
}

impl<const T: i32, Inner: Default> Default for PipelineSubject<T, Inner> {
    fn default() -> Self {
        Self {
            subject_type: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE(T),
            inner: Default::default(),
        }
    }
}

const SUBOBJECT_TYPE_ROOT_SIGNATURE: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_ROOT_SIGNATURE.0;
// const SUBOBJECT_TYPE_VS: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_VS.0;
const SUBOBJECT_TYPE_PS: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_PS.0;
// const SUBOBJECT_TYPE_DS: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DS.0;
// const SUBOBJECT_TYPE_HS: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_HS.0;
// const SUBOBJECT_TYPE_GS: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_GS.0;
// const SUBOBJECT_TYPE_CS: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_CS.0;
// const SUBOBJECT_TYPE_STREAM_OUTPUT: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_STREAM_OUTPUT.0;
const SUBOBJECT_TYPE_BLEND: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_BLEND.0;
const SUBOBJECT_TYPE_SAMPLE_MASK: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_SAMPLE_MASK.0;
const SUBOBJECT_TYPE_RASTERIZER: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_RASTERIZER.0;
const SUBOBJECT_TYPE_DEPTH_STENCIL: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DEPTH_STENCIL.0;
// const SUBOBJECT_TYPE_INPUT_LAYOUT: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_INPUT_LAYOUT.0;
// const SUBOBJECT_TYPE_IB_STRIP_CUT_VALUE: i32 =
// D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_IB_STRIP_CUT_VALUE.0;
const SUBOBJECT_TYPE_PRIMITIVE_TOPOLOGY: i32 =
    D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_PRIMITIVE_TOPOLOGY.0;
const SUBOBJECT_TYPE_RENDER_TARGET_FORMATS: i32 =
    D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_RENDER_TARGET_FORMATS.0;
const SUBOBJECT_TYPE_DEPTH_STENCIL_FORMAT: i32 =
    D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DEPTH_STENCIL_FORMAT.0;
const SUBOBJECT_TYPE_SAMPLE_DESC: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_SAMPLE_DESC.0;
// const SUBOBJECT_TYPE_NODE_MASK: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_NODE_MASK.0;
// const SUBOBJECT_TYPE_CACHED_PSO: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_CACHED_PSO.0;
// const SUBOBJECT_TYPE_FLAGS: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_FLAGS.0;
// const SUBOBJECT_TYPE_DEPTH_STENCIL1: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DEPTH_STENCIL1.0;
// const SUBOBJECT_TYPE_VIEW_INSTANCING: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_VIEW_INSTANCING.0;
const SUBOBJECT_TYPE_AS: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_AS.0;
const SUBOBJECT_TYPE_MS: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_MS.0;
// const SUBOBJECT_TYPE_DEPTH_STENCIL2: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DEPTH_STENCIL2.0;
// const SUBOBJECT_TYPE_RASTERIZER1: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_RASTERIZER1.0;
// const SUBOBJECT_TYPE_MAX_VALID: i32 = D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_MAX_VALID.0;

#[repr(C)]
#[derive(Default)]
struct MeshShaderPipelineStateDesc {
    pub root_signature: PipelineSubject<
        SUBOBJECT_TYPE_ROOT_SIGNATURE,
        ::std::mem::ManuallyDrop<::core::option::Option<ID3D12RootSignature>>,
    >,
    pub r#as: PipelineSubject<SUBOBJECT_TYPE_AS, D3D12_SHADER_BYTECODE>,
    pub ms: PipelineSubject<SUBOBJECT_TYPE_MS, D3D12_SHADER_BYTECODE>,
    pub ps: PipelineSubject<SUBOBJECT_TYPE_PS, D3D12_SHADER_BYTECODE>,
    pub blend: PipelineSubject<SUBOBJECT_TYPE_BLEND, D3D12_BLEND_DESC>,
    pub sample_mask: PipelineSubject<SUBOBJECT_TYPE_SAMPLE_MASK, u32>,
    pub rasterizer: PipelineSubject<SUBOBJECT_TYPE_RASTERIZER, D3D12_RASTERIZER_DESC>,
    pub depth_stencil: PipelineSubject<SUBOBJECT_TYPE_DEPTH_STENCIL, D3D12_DEPTH_STENCIL_DESC>,
    pub primitive_topology:
        PipelineSubject<SUBOBJECT_TYPE_PRIMITIVE_TOPOLOGY, D3D12_PRIMITIVE_TOPOLOGY_TYPE>,
    pub rtv_formats: PipelineSubject<SUBOBJECT_TYPE_RENDER_TARGET_FORMATS, D3D12_RT_FORMAT_ARRAY>,
    pub dsv_format: PipelineSubject<SUBOBJECT_TYPE_DEPTH_STENCIL_FORMAT, DXGI_FORMAT>,
    pub sample_desc: PipelineSubject<SUBOBJECT_TYPE_SAMPLE_DESC, DXGI_SAMPLE_DESC>,
}
