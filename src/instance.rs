pub use internal::{load, NDIHandle};

#[cfg(not(feature = "dynamic-link"))]
mod internal {
    use crate::{sdk, NDIInstance};
    use std::ops::Deref;
    use std::sync::Arc;

    unsafe impl Send for NDIHandle {}
    pub struct NDIHandle {
        instance: sdk::NDIlib_v3,
    }
    impl Deref for NDIHandle {
        type Target = sdk::NDIlib_v3;

        fn deref(&self) -> &sdk::NDIlib_v3 {
            &self.instance
        }
    }

    pub fn load() -> Result<NDIInstance, String> {
        let instance = unsafe { sdk::NDIlib_v3_load().as_ref() };
        match instance {
            None => Err("Failed to load lib".to_string()),
            Some(inst) => Ok(NDIInstance {
                handle: Arc::new(NDIHandle { instance: *inst }),
            }),
        }
    }
}

#[cfg(feature = "dynamic-link")]
mod internal {
    use crate::{sdk, NDIInstance};
    use libloading::{Library, Symbol};
    use std::env;
    use std::ops::Deref;
    use std::path::Path;
    use std::sync::Arc;

    unsafe impl Send for NDIHandle {}
    pub struct NDIHandle {
        _handle: Option<Library>, // TODO - remove this when static?
        instance: sdk::NDIlib_v3,
    }
    impl Deref for NDIHandle {
        type Target = sdk::NDIlib_v3;

        fn deref(&self) -> &sdk::NDIlib_v3 {
            &self.instance
        }
    }

    pub fn load(custom_path: Option<String>) -> Result<NDIInstance, String> {
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
            Ok(lib) => unsafe {
                let symbol: std::io::Result<Symbol<fn() -> *const sdk::NDIlib_v3>> =
                    lib.get(b"NDIlib_v3_load");
                match symbol {
                    Err(e) => Err(format!("Invalid lib: {}", e)),
                    Ok(s) => {
                        let instance = s();
                        if instance.is_null() {
                            Err("Library failed to initialise".to_string())
                        } else {
                            Ok(NDIInstance {
                                handle: Arc::new(NDIHandle {
                                    _handle: Some(lib),
                                    instance: *instance,
                                }),
                            })
                        }
                    }
                }
            },
        }
    }
}
