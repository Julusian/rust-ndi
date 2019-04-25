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
pub mod send;
mod util;

pub use crate::instance::load;
use crate::receive::{ReceiveBandwidth, ReceiveColorFormat, ReceiveCreateError, ReceiveInstance};
use crate::send::{SendCreateError, SendInstance};

pub struct NDIInstance {
    handle: Arc<NDIHandle>,
}
impl NDIInstance {
    pub fn create_find_instance(&self, show_local_sources: bool) -> Option<FindInstance> {
        finder::create_find_instance(self.handle.clone(), show_local_sources)
    }
    pub fn create_receive_instance(
        &self,
        bandwidth: ReceiveBandwidth,
        color_format: ReceiveColorFormat,
    ) -> Result<ReceiveInstance, ReceiveCreateError> {
        receive::create_receive_instance(self.handle.clone(), bandwidth, color_format)
    }

    pub fn create_send_instance(
        &self,
        name: String,
        clock_video: bool,
        clock_audio: bool,
    ) -> Result<SendInstance, SendCreateError> {
        send::create_send_instance(self.handle.clone(), name, clock_video, clock_audio)
    }
}
