use crate::finder::FindInstance;
use crate::instance::NDIHandle;
use std::sync::Arc;

#[allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    clippy::all
)]
mod sdk;

pub mod finder;
mod instance;
pub mod receive;
mod util;

pub use instance::load;

pub struct NDIInstance {
    handle: Arc<NDIHandle>,
}
impl NDIInstance {
    pub fn init(&self) -> bool {
        unsafe { self.handle.NDIlib_initialize.unwrap()() }
    }
    pub fn destroy(&self) {
        unsafe { self.handle.NDIlib_destroy.unwrap()() }
    }
    pub fn create_find_instance(&self, show_local_sources: bool) -> Option<FindInstance> {
        finder::create_find_instance(self.handle.clone(), show_local_sources)
    }
}
