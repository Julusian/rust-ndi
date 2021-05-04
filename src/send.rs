use crate::{sdk, NDIHandle};
use std::ffi::CString;
use std::ptr::{null, null_mut};
use std::sync::Arc;

unsafe impl Send for SendInstance {}
pub struct SendInstance {
    sdk_instance: Arc<NDIHandle>,
    instance: sdk::NDIlib_send_instance_t,
    in_flight_video: Option<NDISendVideoFrame>,
}
impl Drop for SendInstance {
    fn drop(&mut self) {
        unsafe {
            if self.in_flight_video.is_some() {
                self.send_video_flush();
            }

            self.sdk_instance.NDIlib_send_destroy.unwrap()(self.instance);
        }
    }
}
impl SendInstance {
    pub fn send_video(&mut self, frame: NDISendVideoFrame) {
        unsafe {
            // TODO - is this going to be a race condition?
            //            self.in_flight_video = Some(frame);
            //            self.sdk_instance.NDIlib_send_send_video_v2.unwrap()(self.instance, &self.in_flight_video.as_ref().unwrap().instance);
            self.sdk_instance.NDIlib_send_send_video_v2.unwrap()(self.instance, &frame.instance);
            self.in_flight_video = Some(frame);
        }
    }
    pub fn send_video_async(&mut self, frame: NDISendVideoFrame) {
        unsafe {
            self.sdk_instance.NDIlib_send_send_video_async_v2.unwrap()(self.instance, &frame.instance);
            self.in_flight_video = Some(frame);
        }
    }
    pub fn send_video_flush(&mut self) {
        unsafe {
            self.sdk_instance.NDIlib_send_send_video_async_v2.unwrap()(self.instance, null());
            self.in_flight_video = None;
        }
    }
    pub fn send_audio(&mut self, frame: NDISendAudioFrame) {
        unsafe {
            self.sdk_instance.NDIlib_send_send_audio_v2.unwrap()(self.instance, &frame.instance);
        }
    }
}

pub enum FrameFormatType {
    Progressive = sdk::NDIlib_frame_format_type_progressive as isize,
    Interleaved = sdk::NDIlib_frame_format_type_interleaved as isize,
    Field0 = sdk::NDIlib_frame_format_type_field_0 as isize,
    Field1 = sdk::NDIlib_frame_format_type_field_1 as isize,
}

#[derive(Debug)]
pub enum SendColorFormat {
    Uyvy = sdk::NDIlib_FourCC_type_UYVY as isize,
    Yv12 = sdk::NDIlib_FourCC_type_YV12 as isize,
    Nv12 = sdk::NDIlib_FourCC_type_NV12 as isize,
    I420 = sdk::NDIlib_FourCC_type_I420 as isize,
    Bgra = sdk::NDIlib_FourCC_type_BGRA as isize,
    Bgrx = sdk::NDIlib_FourCC_type_BGRX as isize,
    Rgba = sdk::NDIlib_FourCC_type_RGBA as isize,
    Rgbx = sdk::NDIlib_FourCC_type_RGBX as isize,
    Uyva = sdk::NDIlib_FourCC_type_UYVA as isize,
}

pub struct NDISendVideoFrameBuilder {
    instance: sdk::NDIlib_video_frame_v2_t,
    metadata: Option<String>,
    data: Vec<u8>,
}
impl NDISendVideoFrameBuilder {
    pub fn with_framerate(mut self, num: i32, den: i32) -> Self {
        self.instance.frame_rate_N = num;
        self.instance.frame_rate_D = den;
        self
    }
    pub fn with_aspect_ratio(mut self, aspect_ratio: f32) -> Self {
        self.instance.picture_aspect_ratio = aspect_ratio;
        self
    }
    pub fn with_timecode(mut self, timecode: i64) -> Self {
        self.instance.timecode = timecode;
        self
    }
    pub fn with_data(mut self, data: Vec<u8>, line_stride: i32, format: SendColorFormat) -> Self {
        self.data = data;
        self.instance.line_stride_in_bytes = line_stride;
        self.instance.FourCC = format as u32;
        self
    }
    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.metadata = Some(metadata);
        self
    }
    pub fn with_timestamp(mut self, timestamp: i64) -> Self {
        self.instance.timestamp = timestamp;
        self
    }
    pub fn build(self) -> Result<NDISendVideoFrame, SendCreateError> {
        // TODO - change return error type
        let mut res = NDISendVideoFrame {
            instance: self.instance,
            metadata: self.metadata,
            data: self.data,
        };

        if let Some(metadata) = &res.metadata {
            res.instance.p_metadata = CString::new(metadata.as_bytes())
                .map_err(|_| SendCreateError::InvalidName)?
                .as_ptr();
        }

        res.data
            .resize((res.instance.line_stride_in_bytes * res.instance.yres) as usize, 0);
        res.instance.p_data = res.data.as_mut_ptr();

        Ok(res)
    }
}

pub struct NDISendAudioFrameBuilder {
    instance: sdk::NDIlib_audio_frame_v2_t,
    data: Vec<f32>,
}

impl NDISendAudioFrameBuilder {
    pub fn with_timecode(mut self, timecode: i64) -> Self {
        self.instance.timecode = timecode;
        self
    }
    pub fn with_data(mut self, data: Vec<f32>, sample_count: i32) -> Self {
        self.data = data;
        self.instance.no_samples = sample_count;
        self.instance.channel_stride_in_bytes = (self.instance.no_samples)*4;
        self
    }
    pub fn with_timestamp(mut self, timestamp: i64) -> Self {
        self.instance.timestamp = timestamp;
        self
    }
    pub fn build(self) -> Result<NDISendAudioFrame, SendCreateError> {
        // TODO - change return error type
        let mut res = NDISendAudioFrame {
            instance: self.instance,
            data: self.data,
        };
        res.instance.p_data = res.data.as_mut_ptr();
        Ok(res)
    }
}
pub fn create_ndi_send_video_frame(width: i32, height: i32, frame_type: FrameFormatType) -> NDISendVideoFrameBuilder {
    NDISendVideoFrameBuilder {
        instance: sdk::NDIlib_video_frame_v2_t {
            xres: width,
            yres: height,
            FourCC: sdk::NDIlib_FourCC_type_BGRA,
            frame_rate_N: 0,
            frame_rate_D: 0,
            picture_aspect_ratio: 0.0,
            frame_format_type: frame_type as u32,
            timecode: sdk::NDIlib_send_timecode_synthesize,
            p_data: null_mut(),
            line_stride_in_bytes: 0,
            p_metadata: null(),
            timestamp: 0,
        },
        metadata: None,
        data: vec![],
    }
}

pub fn create_ndi_send_audio_frame(channel_count: i32, sample_rate: i32) -> NDISendAudioFrameBuilder {
    NDISendAudioFrameBuilder {
        instance: sdk::NDIlib_audio_frame_v2_t {
            sample_rate,
            no_channels: channel_count,
            no_samples: 0,
            timecode: sdk::NDIlib_send_timecode_synthesize,
            channel_stride_in_bytes: 0,
            p_data: null_mut(),
            p_metadata: null(),
            timestamp: 0,
        },
        data: vec![],
    }
}

pub struct NDISendVideoFrame {
    instance: sdk::NDIlib_video_frame_v2_t,
    metadata: Option<String>,
    data: Vec<u8>,
}

pub struct NDISendAudioFrame {
    instance: sdk::NDIlib_audio_frame_v2_t,
    data: Vec<f32>,
}

#[derive(Debug)]
pub enum SendCreateError {
    InvalidName,
    Failed,
}

pub fn create_send_instance(
    sdk_instance: Arc<NDIHandle>,
    name: String,
    clock_video: bool,
    clock_audio: bool,
) -> Result<SendInstance, SendCreateError> {
    let name2 = CString::new(name.as_bytes()).map_err(|_| SendCreateError::InvalidName)?;

    let props = sdk::NDIlib_send_create_t {
        p_ndi_name: name2.as_ptr(),
        p_groups: null(),
        clock_video,
        clock_audio,
    };

    let instance = unsafe { sdk_instance.NDIlib_send_create.unwrap()(&props) };

    if instance.is_null() {
        Err(SendCreateError::Failed)
    } else {
        Ok(SendInstance {
            sdk_instance,
            instance,
            in_flight_video: None,
        })
    }
}
