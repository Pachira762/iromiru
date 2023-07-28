use windows::Win32::Graphics::{
    Direct3D::{Dxc::*, *},
    Direct3D12::*,
};

pub enum Blob {
    D3D(Option<ID3DBlob>),
    Dxc(Option<IDxcBlob>),
}

impl Blob {
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            match self {
                Blob::D3D(Some(blob)) => std::slice::from_raw_parts(
                    blob.GetBufferPointer() as *const u8,
                    blob.GetBufferSize() as usize,
                ),
                Blob::Dxc(Some(blob)) => std::slice::from_raw_parts(
                    blob.GetBufferPointer() as *const u8,
                    blob.GetBufferSize() as usize,
                ),
                _ => panic!("is not Some Blob."),
            }
        }
    }

    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.as_slice()) }
    }

    pub fn as_bytecode(&self) -> D3D12_SHADER_BYTECODE {
        unsafe {
            match self {
                Blob::Dxc(Some(blob)) => D3D12_SHADER_BYTECODE {
                    pShaderBytecode: blob.GetBufferPointer(),
                    BytecodeLength: blob.GetBufferSize(),
                },
                _ => panic!("is not Some IDxcBlob."),
            }
        }
    }

    pub fn d3d_option(&mut self) -> &mut Option<ID3DBlob> {
        match self {
            Blob::D3D(opt) => opt,
            _ => panic!("is not D3D Blob."),
        }
    }

    pub fn dxc_option(&mut self) -> &mut Option<IDxcBlob> {
        match self {
            Blob::Dxc(opt) => opt,
            _ => panic!("is not Dxc Blob."),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Blob::D3D(None) | Blob::Dxc(None) => true,
            Blob::D3D(Some(blob)) => unsafe { blob.GetBufferSize() == 0 },
            Blob::Dxc(Some(blob)) => unsafe { blob.GetBufferSize() == 0 },
        }
    }
}
