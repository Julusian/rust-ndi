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

/// A loaded SDK Instance
pub struct NDIInstance {
    handle: Arc<NDIHandle>,
}
impl NDIInstance {
    /// Initialise an instance of the NDI source finder
    ///
    /// # Arguments
    ///
    /// * `show_local_sources` Whether to include sources from the local machine
    ///
    /// # Returns
    ///
    /// An instance if it was successful, or None if the SDK failed
    ///
    pub fn create_find_instance(&self, show_local_sources: bool) -> Option<FindInstance> {
        finder::create_find_instance(self.handle.clone(), show_local_sources)
    }

    /// Initialise an instance of the NDI receiver
    pub fn create_receive_instance(
        &self,
        bandwidth: ReceiveBandwidth,
        color_format: ReceiveColorFormat,
    ) -> Result<Arc<ReceiveInstance>, ReceiveCreateError> {
        receive::create_receive_instance(self.handle.clone(), bandwidth, color_format)
    }

    /// Initialise an instance of the NDI sender
    pub fn create_send_instance(
        &self,
        name: String,
        clock_video: bool,
        clock_audio: bool,
    ) -> Result<SendInstance, SendCreateError> {
        send::create_send_instance(self.handle.clone(), name, clock_video, clock_audio)
    }
}
