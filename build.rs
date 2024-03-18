#[cfg(feature = "buildtime_bindgen")]
fn build() {
    use std::path::PathBuf;

    let config = pkg_config::Config::new()
        .probe("libdrm")
        .unwrap()
        .include_paths;

    let config: Vec<String> = config
        .iter()
        .map(|path| format!("-I{}", path.to_str().unwrap()))
        .collect();

    let out_path = PathBuf::from("./bindings/");

    {
        let bindings = bindgen::Builder::default()
            // Do not generate unstable Rust code that
            // requires a nightly rustc and enabling
            // unstable features.
            // .no_unstable_rust()
            // The input header we would like to generate
            // bindings for.
            .header("wrapper/wrapper_drm.h")
            .header("wrapper/wrapper_amdgpu.h")
            .header("wrapper/wrapper_gpu_metrics.h")
            .clang_args(config.iter())
            .use_core()
            .ctypes_prefix("::core::ffi")
            // .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            // Finish the builder and generate the bindings.
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate bindings: {wrapper_name}");

        bindings
            .write_to_file(out_path.join("drm.rs"))
            .expect("Couldn't write bindings!");
    }

    {
        let smu_v11_0_7_ppt = bindgen::Builder::default()
            .header("wrapper/wrapper_atomfirmware.h")
            .header("wrapper/smu11_driver_if_sienna_cichlid.h")
            .header("wrapper/smu_v11_0_7_pptable.h")
            .clang_args(config.iter())
            .use_core()
            .ctypes_prefix("::core::ffi")
            .generate()
            .expect("Unable to generate bindings: {wrapper_name}");

        smu_v11_0_7_ppt
            .write_to_file(out_path.join("ppt").join("smu_v11_0_7_ppt.rs"))
            .expect("Couldn't write bindings!");
    }

    {
        let smu_v13_0_0_ppt = bindgen::Builder::default()
            .header("wrapper/wrapper_atomfirmware.h")
            .header("wrapper/smu13_driver_if_v13_0_0.h")
            .header("wrapper/smu_v13_0_0_pptable.h")
            .clang_args(config.iter())
            .use_core()
            .ctypes_prefix("::core::ffi")
            .generate()
            .expect("Unable to generate bindings: {wrapper_name}");

        smu_v13_0_0_ppt
            .write_to_file(out_path.join("ppt").join("smu_v13_0_0_ppt.rs"))
            .expect("Couldn't write bindings!");
    }

    convert_amdgpu_ids();
}

#[cfg(feature = "buildtime_bindgen")]
fn convert_amdgpu_ids() {
    use std::fmt::Write;
    const AMDGPU_IDS: &str = include_str!("bindings/amdgpu.ids");

    let mut s = String::from("pub const AMDGPU_IDS: &[(u32, u32, &str)] = &[\n");

    for line in AMDGPU_IDS.lines() {
        if line.starts_with('#') { continue }

        let mut split = line.split(",\t");

        if let [Some(did), Some(rid), Some(name)] = [split.next(), split.next(), split.next()] {
            writeln!(s, "    (0x{did}, 0x{rid}, {name:?}),").unwrap();
        }
    }

    writeln!(s, "];").unwrap();

    std::fs::write("bindings/amdgpu_ids.rs", s).unwrap();
}

fn main() {
    #[cfg(feature = "link-drm")]
    println!("cargo:rustc-link-lib=drm");
    #[cfg(feature = "link-drm")]
    println!("cargo:rustc-link-lib=drm_amdgpu");

    #[cfg(feature = "buildtime_bindgen")]
    build();
}
