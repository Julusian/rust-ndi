use crate::finder::FindSource;
use crate::util::to_ndi_source;
use crate::{sdk, NDIHandle};
use ptrplus::AsPtr;
use std::collections::HashMap;
use std::ops::Deref;
use std::ptr::{null, null_mut};
use std::slice;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};

pub struct GuardedPointer<'a, T, T2> {
    _guard: MutexGuard<'a, T>,
    value: &'a [T2],
}
impl<'a, T, T2> Deref for GuardedPointer<'a, T, T2> {
    type Target = [T2];

    fn deref(&self) -> &[T2] {
        self.value
    }
}

pub type VideoFrameData<'a> = GuardedPointer<'a, sdk::NDIlib_video_frame_v2_t, u8>;
unsafe impl Send for VideoFrame {}
unsafe impl Sync for VideoFrame {}
pub struct VideoFrame {
    id: usize,
    instance: Arc<Mutex<sdk::NDIlib_video_frame_v2_t>>,
    parent: Weak<ReceiveInstance>,

    pub width: i32,
    pub height: i32,

    pub frame_rate_n: i32,
    pub frame_rate_d: i32,
    //    pub FourCC: NDIlib_FourCC_type_e,
    //    pub picture_aspect_ratio: f32,
    //    pub frame_format_type: NDIlib_frame_format_type_e,
    pub timecode: i64,
    //    pub p_data: *mut u8,
    //    pub line_stride_in_bytes: ::std::os::raw::c_int,
    //    pub p_metadata: *const ::std::os::raw::c_char,
    pub timestamp: i64,
}
impl Drop for VideoFrame {
    fn drop(&mut self) {
        if let Some(parent) = self.parent.upgrade() {
            parent.free_video(self.id);
        }
    }
}
impl VideoFrame {
    pub fn lock_data(&self) -> Option<VideoFrameData> {
        if let Ok(locked) = self.instance.lock() {
            unsafe {
                let len = locked.line_stride_in_bytes * locked.yres;
                let data = slice::from_raw_parts(locked.p_data, len as usize);
                Some(GuardedPointer {
                    _guard: locked,
                    value: data,
                })
            }
        } else {
            None
        }
    }
}

pub type AudioFrameData<'a> = GuardedPointer<'a, sdk::NDIlib_audio_frame_v2_t, f32>;
unsafe impl Send for AudioFrame {}
unsafe impl Sync for AudioFrame {}
pub struct AudioFrame {
    id: usize,
    instance: Arc<Mutex<sdk::NDIlib_audio_frame_v2_t>>,
    parent: Weak<ReceiveInstance>,

    pub sample_rate: i32,
    pub channel_count: i32,
    pub sample_count: i32,
    pub timecode: i64,
    //    pub p_data: *mut f32,
    //    pub channel_stride_in_bytes: ::std::os::raw::c_int,
    //    pub p_metadata: *const ::std::os::raw::c_char,
    pub timestamp: i64,
}
impl Drop for AudioFrame {
    fn drop(&mut self) {
        if let Some(parent) = self.parent.upgrade() {
            parent.free_audio(self.id);
        }
    }
}
impl AudioFrame {
    pub fn lock_data(&self) -> Option<AudioFrameData> {
        if let Ok(locked) = self.instance.lock() {
            unsafe {
                // Divide by four as this is a list of f32
                let len = locked.channel_stride_in_bytes * locked.no_channels / 4; 
                let data = slice::from_raw_parts(locked.p_data, len as usize);
                Some(GuardedPointer {
                    _guard: locked,
                    value: data,
                })
            }
        } else {
            None
        }
    }
}

struct ReceiveDataStore<T> {
    data: Mutex<HashMap<usize, Arc<Mutex<T>>>>,
    next_id: AtomicUsize,
}
impl<T> ReceiveDataStore<T> {
    fn remove(&self, id: usize) -> Option<Arc<Mutex<T>>> {
        if let Ok(mut data_store) = self.data.lock() {
            if let Some(data) = data_store.remove(&id) {
                Some(data)
            } else {
                None
            }
        } else {
            None
        }
    }
    fn track(&self, data: T) -> Option<(usize, Arc<Mutex<T>>)> {
        let video2 = Arc::new(Mutex::new(data));

        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut frame_list) = self.data.lock() {
            frame_list.insert(id, video2.clone());
            Some((id, video2))
        } else {
            None
        }
    }
}

unsafe impl Send for ReceiveInstance {}
unsafe impl Sync for ReceiveInstance {} // TODO - is this true? what is safety of methods on instance like?
pub struct ReceiveInstance {
    sdk_instance: Arc<NDIHandle>,
    instance: sdk::NDIlib_recv_instance_t,
    video_frames: ReceiveDataStore<sdk::NDIlib_video_frame_v2_t>,
    audio_frames: ReceiveDataStore<sdk::NDIlib_audio_frame_v2_t>,
}
impl Drop for ReceiveInstance {
    fn drop(&mut self) {
        unsafe {
            if let Ok(frame_store) = self.video_frames.data.lock() {
                for f in frame_store.values() {
                    self.free_video_inner(f)
                }
            }
            if let Ok(frame_store) = self.audio_frames.data.lock() {
                for f in frame_store.values() {
                    self.free_audio_inner(f)
                }
            }

            self.sdk_instance.NDIlib_recv_destroy.unwrap()(self.instance);
        }
    }
}
impl ReceiveInstance {
    pub fn connect(&self, source: Option<&FindSource>) -> bool {
        match source {
            None => unsafe {
                self.sdk_instance.NDIlib_recv_connect.unwrap()(self.instance, null());
                true
            },
            Some(s) => {
                if let Ok(s2) = to_ndi_source(s) {
                    unsafe {
                        self.sdk_instance.NDIlib_recv_connect.unwrap()(self.instance, &s2.2);
                    }

                    true
                } else {
                    false
                }
            }
        }
    }
    fn free_video(&self, id: usize) {
        if let Some(frame) = self.video_frames.remove(id) {
            self.free_video_inner(&frame);
        }
    }
    fn free_video_inner(&self, video: &Arc<Mutex<sdk::NDIlib_video_frame_v2_t>>) {
        if let Ok(mut ndi_ref) = video.lock() {
            unsafe {
                self.sdk_instance.NDIlib_recv_free_video_v2.unwrap()(self.instance, &*ndi_ref);
                ndi_ref.p_data = null_mut();
            }
        } else {
            // TODO - ?
        }
    }
    fn free_audio(&self, id: usize) {
        if let Some(frame) = self.audio_frames.remove(id) {
            self.free_audio_inner(&frame);
        }
    }
    fn free_audio_inner(&self, audio: &Arc<Mutex<sdk::NDIlib_audio_frame_v2_t>>) {
        if let Ok(mut ndi_ref) = audio.lock() {
            unsafe {
                self.sdk_instance.NDIlib_recv_free_audio_v2.unwrap()(self.instance, &*ndi_ref);
                ndi_ref.p_data = null_mut();
            }
        } else {
            // TODO - ?
        }
    }
}

#[derive(Debug)]
pub enum ReceiveCaptureError {
    Failed, // TODO
    Poisoned,
    Invalid,
}

#[derive(Debug)]
pub enum ReceiveCaptureResultType {
    None,
    Video,
    Audio,
    Metadata,
    Error,
    StatusChange,
}
pub enum ReceiveCaptureResult {
    None,
    Video(VideoFrame),
    Audio(AudioFrame),
    Metadata(u32),
}

pub trait ReceiveInstanceExt {
    fn receive_capture(
        &self,
        capture_video: bool,
        capture_audio: bool,
        capture_metadata: bool,
        timeout: u32,
    ) -> Result<ReceiveCaptureResult, ReceiveCaptureError>;
}

impl ReceiveInstanceExt for Arc<ReceiveInstance> {
    fn receive_capture(
        &self,
        capture_video: bool,
        capture_audio: bool,
        capture_metadata: bool,
        timeout: u32,
    ) -> Result<ReceiveCaptureResult, ReceiveCaptureError> {
        let video_data = if capture_video {
            Some(sdk::NDIlib_video_frame_v2_t {
                xres: 0,
                yres: 0,
                FourCC: Default::default(),
                frame_rate_N: 0,
                frame_rate_D: 0,
                picture_aspect_ratio: 0.0,
                frame_format_type: Default::default(),
                timecode: 0,
                p_data: null_mut(),
                line_stride_in_bytes: 0,
                p_metadata: null(),
                timestamp: 0,
            })
        } else {
            None
        };
        let audio_data = if capture_audio {
            Some(sdk::NDIlib_audio_frame_v2_t {
                sample_rate: 0,
                no_channels: 0,
                no_samples: 0,
                timecode: 0,
                p_data: null_mut(),
                channel_stride_in_bytes: 0,
                p_metadata: null(),
                timestamp: 0,
            })
        } else {
            None
        };
        let metadata = if capture_metadata {
            Some(sdk::NDIlib_metadata_frame_t {
                length: 0,
                timecode: 0,
                p_data: null_mut(),
            })
        } else {
            None
        };

        let captured = unsafe {
            self.sdk_instance.NDIlib_recv_capture_v2.unwrap()(
                self.instance,
                video_data.as_ref().as_ptr() as *mut sdk::NDIlib_video_frame_v2_t,
                audio_data.as_ref().as_ptr() as *mut sdk::NDIlib_audio_frame_v2_t,
                metadata.as_ref().as_ptr() as *mut sdk::NDIlib_metadata_frame_t,
                timeout,
            )
        };
        match captured {
            sdk::NDIlib_frame_type_video => match video_data {
                None => Err(ReceiveCaptureError::Failed),
                Some(video_data) => match self.video_frames.track(video_data) {
                    None => Err(ReceiveCaptureError::Poisoned),
                    Some(v) => {
                        let frame = VideoFrame {
                            id: v.0,
                            instance: v.1,
                            parent: Arc::downgrade(self),

                            width: video_data.xres,
                            height: video_data.yres,

                            frame_rate_d: video_data.frame_rate_D,
                            frame_rate_n: video_data.frame_rate_N,

                            timecode: video_data.timecode,

                            timestamp: video_data.timestamp,
                        };
                        Ok(ReceiveCaptureResult::Video(frame))
                    }
                },
            },
            sdk::NDIlib_frame_type_audio => match audio_data {
                None => Err(ReceiveCaptureError::Failed),
                Some(audio_data) => match self.audio_frames.track(audio_data) {
                    None => Err(ReceiveCaptureError::Poisoned),
                    Some(v) => {
                        let frame = AudioFrame {
                            id: v.0,
                            instance: v.1,
                            parent: Arc::downgrade(self),

                            sample_rate: audio_data.sample_rate,
                            channel_count: audio_data.no_channels,
                            sample_count: audio_data.no_samples,
                            timecode: audio_data.timecode,
                            timestamp: audio_data.timestamp,
                        };
                        Ok(ReceiveCaptureResult::Audio(frame))
                    }
                },
            },
            sdk::NDIlib_frame_type_none => Ok(ReceiveCaptureResult::None),
            _ => Err(ReceiveCaptureError::Invalid),
        }
    }
}

#[derive(Debug)]
pub enum ReceiveCreateError {
    Failed,
}

#[derive(Debug)]
pub enum ReceiveBandwidth {
    MetadataOnly = sdk::NDIlib_recv_bandwidth_metadata_only as isize,
    AudioOnly = sdk::NDIlib_recv_bandwidth_audio_only as isize,
    Lowest = sdk::NDIlib_recv_bandwidth_lowest as isize,
    Highest = sdk::NDIlib_recv_bandwidth_highest as isize,
}

#[derive(Debug)]
pub enum ReceiveColorFormat {
    Fastest = sdk::NDIlib_recv_color_format_fastest as isize,
    BgrxBgra = sdk::NDIlib_recv_color_format_BGRX_BGRA as isize, // No alpha channel: BGRX, Alpha channel: BGRA
    UyvyBgra = sdk::NDIlib_recv_color_format_UYVY_BGRA as isize, // No alpha channel: UYVY, Alpha channel: BGRA
    RgbxRgba = sdk::NDIlib_recv_color_format_RGBX_RGBA as isize, // No alpha channel: RGBX, Alpha channel: RGBA
    UyvyRgba = sdk::NDIlib_recv_color_format_UYVY_RGBA as isize, // No alpha channel: UYVY, Alpha channel: RGBA
}

pub fn create_receive_instance(
    sdk_instance: Arc<NDIHandle>,
    bandwidth: ReceiveBandwidth,
    color_format: ReceiveColorFormat,
) -> Result<Arc<ReceiveInstance>, ReceiveCreateError> {
    let props = sdk::NDIlib_recv_create_v3_t {
        source_to_connect_to: sdk::NDIlib_source_t {
            p_ndi_name: null(),
            __bindgen_anon_1: sdk::NDIlib_source_t__bindgen_ty_1 { p_url_address: null() },
        },
        color_format: color_format as u32,
        bandwidth: bandwidth as i32,
        allow_video_fields: false,
        p_ndi_recv_name: null(),
    };

    let instance = unsafe { sdk_instance.NDIlib_recv_create_v3.unwrap()(&props) };

    if instance.is_null() {
        Err(ReceiveCreateError::Failed)
    } else {
        Ok(Arc::new(ReceiveInstance {
            sdk_instance,
            instance,
            video_frames: ReceiveDataStore {
                data: Mutex::new(HashMap::new()),
                next_id: AtomicUsize::new(0),
            },
            audio_frames: ReceiveDataStore {
                data: Mutex::new(HashMap::new()),
                next_id: AtomicUsize::new(0),
            },
        }))
    }
}
