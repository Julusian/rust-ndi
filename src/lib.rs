use std::sync::Arc;
use libloading::{Library, Symbol};
use std::path::Path;
use std::env;

#[allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    clippy::all
)]
mod sdk;

pub mod finder;
pub mod receive;
mod util;

pub struct NDIInstance {
    _handle: Library,
    instance: sdk::NDIlib_v3,
}
impl NDIInstance {
    pub fn init(&self) -> bool {
        unsafe {
            self.instance.NDIlib_initialize.unwrap()()
        }
    }
    pub fn destroy(&self) {
        unsafe {
            self.instance.NDIlib_destroy.unwrap()()
        }
    }
}

pub fn load(custom_path: Option<String>) -> Result<Arc<NDIInstance>, String> {
    let lib_path = if let Some(path) = custom_path {
        path
    } else {
        let local_path = "./libndi.so";
        if Path::new(local_path).exists() {
            local_path.to_string()
        } else if let Ok(env_var) = env::var("NDI_RUNTIME_DIR_V3") {
            let p = Path::new(&env_var).join("libndi.so");
            if p.exists() {
                p.to_str().unwrap_or("libndi.so").to_string()
            } else {
                "libndi.so".to_string()
            }
        } else {
            "libndi.so".to_string()
        }
    };

    match Library::new(lib_path) {
        Err(e) => Err(format!("Failed to load lib: {}", e)),
        Ok(lib) => {
            unsafe {
                let symbol: std::io::Result<Symbol<fn() -> *const sdk::NDIlib_v3>> = lib.get(b"NDIlib_v3_load");
                match symbol {
                    Err(e) => Err(format!("Invalid lib: {}", e)),
                    Ok(s) => {
                        let instance = s();
                        if instance.is_null() {
                            Err("Library failed to initialise".to_string())
                        } else {
                            Ok(Arc::new(NDIInstance {
                                _handle: lib,
                                instance: *instance,
                            }))
                        }
                    }
                }
            }
        }
    }
}



//pub fn init() -> bool {
//    unsafe { sdk::NDIlib_initialize() }
//}
//pub fn destroy() {
//    unsafe { sdk::NDIlib_destroy() }
//}
