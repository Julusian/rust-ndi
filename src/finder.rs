use crate::{sdk, NDIInstance};
use std::ffi::CStr;
use std::ptr::null;
use std::slice;
use std::sync::Arc;

#[derive(Clone)]
pub struct FindSource {
    pub name: String,
    pub url: Option<String>,
}

unsafe impl Send for FindInstance {}
pub struct FindInstance {
    sdk_instance: Arc<NDIInstance>,
    instance: sdk::NDIlib_find_instance_t,
}
impl Drop for FindInstance {
    fn drop(&mut self) {
        unsafe {
            self.sdk_instance.instance.NDIlib_find_destroy.unwrap()(self.instance);
        }
    }
}
impl FindInstance {
    pub fn get_current_sources(&self) -> Vec<FindSource> {
        unsafe {
            let mut source_count = 0;
            // Memory is freed on next call, or destroy
            let sources = self.sdk_instance.instance.NDIlib_find_get_current_sources.unwrap()(self.instance, &mut source_count);

            slice::from_raw_parts(sources, source_count as usize)
                .iter()
                .map(|s| {
                    let name = CStr::from_ptr(s.p_ndi_name).to_string_lossy().into_owned();
                    let url = if s.__bindgen_anon_1.p_url_address.is_null() {
                        None
                    } else {
                        Some(
                            CStr::from_ptr(s.__bindgen_anon_1.p_url_address)
                                .to_string_lossy()
                                .into_owned(),
                        )
                    };
                    FindSource { name, url }
                })
                .collect()
        }
    }

    pub fn wait_for_sources(&self, timeout: u32) -> bool {
        unsafe { self.sdk_instance.instance.NDIlib_find_wait_for_sources.unwrap()(self.instance, timeout) }
    }
}

pub fn create_find_instance(sdk_instance: Arc<NDIInstance>, show_local_sources: bool) -> Option<FindInstance> {
    let props = sdk::NDIlib_find_create_t {
        show_local_sources,
        p_groups: null(),
        p_extra_ips: null(),
    };

    let instance = unsafe {
        sdk_instance.instance.NDIlib_find_create_v2.unwrap()(&props) };

    if instance.is_null() {
        None
    } else {
        Some(FindInstance {
            sdk_instance,
            instance,
        })
    }
}
