use std::slice::from_raw_parts;

use windows::core::*;
use windows::Win32::Graphics::Direct3D::Dxc::*;

use super::blob::Blob;

pub struct Compiler {
    util: IDxcUtils,
    compiler: IDxcCompiler3,
    include_handler: IDxcIncludeHandler,
}

impl Compiler {
    pub fn new() -> Result<Self> {
        unsafe {
            let util: IDxcUtils = DxcCreateInstance(&CLSID_DxcLibrary)?;
            let compiler = DxcCreateInstance(&CLSID_DxcCompiler)?;
            let include_handler = util.CreateDefaultIncludeHandler()?;

            Ok(Self {
                util,
                compiler,
                include_handler,
            })
        }
    }

    pub fn compile(
        &self,
        path: PCWSTR,
        entry: PCWSTR,
        target: PCWSTR,
        defines: &[DxcDefine],
    ) -> Result<Blob> {
        unsafe {
            let source = self.util.LoadFile(path, None)?;

            let args = self
                .util
                .BuildArguments(path, entry, target, None, defines)?;

            let result: IDxcResult = self.compiler.Compile(
                &DxcBuffer {
                    Ptr: source.GetBufferPointer(),
                    Size: source.GetBufferSize(),
                    Encoding: DXC_CP_ACP.0,
                },
                Some(from_raw_parts(args.GetArguments(), args.GetCount() as _)),
                &self.include_handler,
            )?;

            let status = result.GetStatus()?;
            if status.is_err() {
                let mut error: Option<IDxcBlobUtf8> = None;
                let mut name = None;
                result.GetOutput(DXC_OUT_ERRORS, &mut name, &mut error)?;

                if error.is_some() && !error.as_ref().unwrap().GetBufferPointer().is_null() {
                    let error = error.unwrap();
                    println!(
                        "{}",
                        std::str::from_utf8(std::slice::from_raw_parts(
                            error.GetBufferPointer() as _,
                            error.GetBufferSize()
                        ))
                        .unwrap()
                    );
                }

                status.ok()?;
            }

            let mut blob = Blob::Dxc(None);
            let mut name = None;
            result.GetOutput(DXC_OUT_OBJECT, &mut name, blob.dxc_option())?;

            Ok(blob)
        }
    }
}
