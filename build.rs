fn _copy_shader_file(file: &str) {
    let from = format!("shaders/{}", file);
    let to = format!(
        "{}/../../../shaders/{}",
        std::env::var("OUT_DIR").unwrap(),
        file
    );

    println!("!cargo:rerun-if-changed={}", from);
    std::fs::copy(from, to).expect("Copy");
}

fn main() {
    //copy_shader_file("color_cloud.hlsl");
    //copy_shader_file("common.hlsl");
    //copy_shader_file("view.hlsl");

    let mut res = winres::WindowsResource::new();

    res.set_manifest(
        r#"
    <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <application>
        <windowsSettings>
            <activeCodePage xmlns="http://schemas.microsoft.com/SMI/2019/WindowsSettings">UTF-8</activeCodePage>
            <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true</dpiAware>
            <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
        </windowsSettings>
    </application>
    <dependency>
        <dependentAssembly>
            <assemblyIdentity
                type="win32"
                name="Microsoft.Windows.Common-Controls"
                version="6.0.0.0"
                processorArchitecture="*"
                publicKeyToken="6595b64144ccf1df"
                language="*"
            />
        </dependentAssembly>
    </dependency>
    </assembly>
    "#,
    );

    res.compile().unwrap();
}
