// build.rs

#[allow(unused_imports)]
use std::io::ErrorKind;
use std::path::Path;
use std::{env, fs};

fn choose_source_dir() -> Option<String> {
    // Follow the 'recommended' install path
    if let Ok(path) = env::var("NDI_RUNTIME_DIR_V5") {
        if Path::new(&path).exists() {
            dbg!("penis");
            dbg!(&path);
            return Some(path);
        }
    }

    // Try the local lib folder
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = Path::new(&dir).join("lib");
    if path.exists() {
        return path.to_str().map(|s| s.to_string());
    }

    #[cfg(target_os = "macos")]
    {
        // Try the standard SDK install location on Mac OS
        let std_location = Path::new("/Library/NDI SDK for macOS/lib/macOS/");
        if std_location.exists() {
            return std_location.to_str().map(|s| s.to_string());
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn main() {
    let source_dir = choose_source_dir();

    // Copy the .dll/.lib files to the deps folder, to make it build
    if let Some(path) = source_dir {
        let source_path = Path::new(&path);
        let dest_path = Path::new(&env::var("OUT_DIR").unwrap()).join("../../../deps");
        fs::copy(
            source_path.join("..\\..\\NewTek NDI 5.0 SDK\\Lib\\x64\\Processing.NDI.Lib.x64.lib"),
            dest_path.join("Processing.NDI.Lib.x64.lib"),
        )
        .expect("copy Processing.NDI.Lib.x64.lib");
        fs::copy(
            source_path.join("Processing.NDI.Lib.x64.dll"),
            dest_path.join("Processing.NDI.Lib.x64.dll"),
        )
        .expect("copy Processing.NDI.Lib.x64.dll");
    }

    if cfg!(not(feature = "dynamic-link")) {
        // Static link against it
        println!("cargo:rustc-link-lib=Processing.NDI.Lib.x64");
    }
}

#[cfg(target_os = "linux")]
fn main() {
    let source_dir = choose_source_dir();

    // Copy the .so files to the deps folder, to make it build
    if let Some(path) = source_dir {
        let source_path = Path::new(&path);
        let dest_path = Path::new(&env::var("OUT_DIR").unwrap()).join("../../../deps");
        fs::copy(source_path.join("libndi.so.5"), dest_path.join("libndi.so.5")).expect("copy libndi.so.5");

        let sl_res = std::os::unix::fs::symlink(Path::new("libndi.so.5"), dest_path.join("libndi.so"));
        if let Err(e) = sl_res {
            if e.kind() != ErrorKind::AlreadyExists {
                panic!("Unknown error: {}", e);
            }
        }
    }

    if cfg!(not(feature = "dynamic-link")) {
        // Static link against it
        println!("cargo:rustc-link-lib=ndi");
    }
}

#[cfg(target_os = "macos")]
fn main() {
    let source_dir = choose_source_dir();
    // Copy the .dylib files to the deps folder, to make it build
    if let Some(path) = source_dir {
        let source_path = Path::new(&path);
        let dest_path = Path::new(&env::var("OUT_DIR").unwrap()).join("../../../deps");
        fs::copy(source_path.join("libndi.dylib"), dest_path.join("libndi.dylib")).expect("copy libndi.dylib");

        let sl_res = std::os::unix::fs::symlink(Path::new("libndi.dylib"), dest_path.join("libndi.dylib"));
        if let Err(e) = sl_res {
            if e.kind() != ErrorKind::AlreadyExists {
                panic!("Unknown error: {}", e);
            }
        }
    }

    if cfg!(not(feature = "dynamic-link")) {
        // Static link against it
        println!("cargo:rustc-link-lib=ndi");
    }
}
