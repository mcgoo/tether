use std::{env, io};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = PathBuf::from(env::var("OUT_DIR")?);
    let rust_path = env::current_dir()?;
    let native_path = rust_path.join("native");

    env::set_current_dir(&native_path)?;

    // Make sure the platform is supported.

    if !cfg!(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos",
    )) {
        panic!("unsupported platform");
    }

    // Link any platform-specific dependencies.

    if cfg!(target_os = "linux") {
        // require gtk 3.14 or later but do not emit metadata to use it -
        // leave that for webkit2gtk
        pkg_config::Config::new()
            .atleast_version("3.14")
            .cargo_metadata(false)
            .probe("gtk+-3.0")?;

        let webkit = pkg_config::Config::new()
            .atleast_version("2.8")
            .probe("webkit2gtk-4.0")?;

        let mut build = cc::Build::new();
        build.file("gtk.c")
            .flag("-ffunction-sections")
            .flag("-fdata-sections")
            .flag("-fPIC")
            .flag("-pthread")
            .flag("-std=c99")
            .flag("-Wall")
            .flag("-Wextra");
        webkit.include_paths.iter().for_each(|inc| { build.include(inc) ; } );
        build.compile("tether");
    } else if cfg!(target_os = "windows") {
        cc::Build::new()
            .file("winapi.cpp")
            .flag("/EHsc")
            .flag("/std:c++17")
            .flag("/W4")
            .compile("tether");
        println!("cargo:rustc-link-lib=dylib=ole32");
        println!("cargo:rustc-link-lib=dylib=user32");
        println!("cargo:rustc-link-lib=dylib=windowsapp");
    } else if cfg!(target_os = "macos") {
        cc::Build::new()
            .file("cocoa.m")
            .flag("-ffunction-sections")
            .flag("-fdata-sections")
            .flag("-fPIC")
            .flag("-ObjC")
            .flag("-fobjc-arc")
            .flag("-Wall")
            .flag("-Wextra")
            .compile("tether");
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=WebKit");
    }
   
    // Generate the bindings to the library.

    bindgen::Builder::default()
        .header("tether.h")
        .generate()
        .map_err(|()| io::Error::new(io::ErrorKind::Other, "bindgen failed"))?
        .write_to_file(out_path.join("bindings.rs"))?;

    Ok(())
}
