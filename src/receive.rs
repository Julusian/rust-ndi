use crate::finder::FindSource;
use crate::sdk;
use crate::util::to_ndi_source;
use ptrplus::AsPtr;
use std::collections::HashMap;
use std::ops::Deref;
use std::ptr::{null, null_mut};
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};

pub type VideoFrameData<'a> = GuardedPointer<'a, sdk::NDIlib_video_frame_v2_t>;
pub struct GuardedPointer<'a, T> {
    _guard: MutexGuard<'a, T>,
    value: &'a [u8],
}
impl<'a, T> Deref for GuardedPointer<'a, T> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.value
    }
}

pub struct VideoFrame {
    id: u64,
    instance: Arc<Mutex<sdk::NDIlib_video_frame_v2_t>>,
    parent: Weak<ReceiveInstance>,

    pub width: i32,
    pub height: i32,

    pub frame_rate_n: i32,
    pub frame_rate_d: i32,
    //    pub FourCC: NDIlib_FourCC_type_e,
    //    pub picture_aspect_ratio: f32,
    //    pub frame_format_type: NDIlib_frame_format_type_e,
    //    pub timecode: i64,
    //    pub p_data: *mut u8,
    //    pub line_stride_in_bytes: ::std::os::raw::c_int,
    //    pub p_metadata: *const ::std::os::raw::c_char,
    //    pub timestamp: i64,
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

pub struct ReceiveInstance {
    instance: sdk::NDIlib_recv_instance_t,
    frames: Mutex<HashMap<u64, Arc<Mutex<sdk::NDIlib_video_frame_v2_t>>>>,
    frame_next_id: AtomicU64,
}
impl Drop for ReceiveInstance {
    fn drop(&mut self) {
        unsafe {
            if let Ok(frame_store) = self.frames.lock() {
                for f in frame_store.values() {
                    self.free_video_inner(f)
                }
            }

            sdk::NDIlib_recv_destroy(self.instance);
        }
    }
}
impl ReceiveInstance {
    pub fn connect(&self, source: Option<&FindSource>) -> bool {
        match source {
            None => unsafe {
                sdk::NDIlib_recv_connect(self.instance, null());
                true
            },
            Some(s) => {
                if let Ok(s2) = to_ndi_source(s) {
                    unsafe {
                        sdk::NDIlib_recv_connect(self.instance, &s2.2);
                    }

                    true
                } else {
                    false
                }
            }
        }
    }
    fn free_video(&self, id: u64) {
        let ndi_frame = if let Ok(mut frame_list) = self.frames.lock() {
            if let Some(frame) = frame_list.remove(&id) {
                Some(frame)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(frame) = ndi_frame {
            self.free_video_inner(&frame);
        }
    }
    fn free_video_inner(&self, video: &Arc<Mutex<sdk::NDIlib_video_frame_v2_t>>) {
        if let Ok(mut ndi_ref) = video.lock() {
            unsafe {
                sdk::NDIlib_recv_free_video_v2(self.instance, &*ndi_ref);
                ndi_ref.p_data = null_mut();
            }
        } else {
            // TODO - ?
        }
    }
    fn track_video(
        &self,
        video: sdk::NDIlib_video_frame_v2_t,
    ) -> Option<(u64, Arc<Mutex<sdk::NDIlib_video_frame_v2_t>>)> {
        let video2 = Arc::new(Mutex::new(video));

        let id = self.frame_next_id.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut frame_list) = self.frames.lock() {
            frame_list.insert(id, video2.clone());
            Some((id, video2))
        } else {
            None
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
    Audio(u32),
    Metadata(u32),
}

pub fn receive_capture(
    recv: &Arc<ReceiveInstance>,
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
    //    let mut vd2 = video_data.unwrap();

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
        sdk::NDIlib_recv_capture_v2(
            recv.instance,
            video_data.as_ref().as_ptr() as *mut sdk::NDIlib_video_frame_v2_t,
            audio_data.as_ref().as_ptr() as *mut sdk::NDIlib_audio_frame_v2_t,
            metadata.as_ref().as_ptr() as *mut sdk::NDIlib_metadata_frame_t,
            timeout,
        )
    };
    match captured {
        sdk::NDIlib_frame_type_video => match video_data {
            None => Err(ReceiveCaptureError::Failed),
            Some(video_data) => match recv.track_video(video_data) {
                None => Err(ReceiveCaptureError::Poisoned),
                Some(v) => {
                    let frame = VideoFrame {
                        id: v.0,
                        instance: v.1,
                        parent: Arc::downgrade(recv),

                        width: video_data.xres,
                        height: video_data.yres,

                        frame_rate_d: video_data.frame_rate_D,
                        frame_rate_n: video_data.frame_rate_N,
                    };
                    Ok(ReceiveCaptureResult::Video(frame))
                }
            },
        },
        sdk::NDIlib_frame_type_audio => Ok(ReceiveCaptureResult::Audio(1)),
        sdk::NDIlib_frame_type_none => Ok(ReceiveCaptureResult::None),
        _ => Err(ReceiveCaptureError::Invalid),
    }
}

#[derive(Debug)]
pub enum CreateReceiveError {
    NulSource,
    Failed,
}

pub fn create_receive_instance() -> Result<ReceiveInstance, CreateReceiveError> {
    let props = sdk::NDIlib_recv_create_v3_t {
        source_to_connect_to: sdk::NDIlib_source_t {
            p_ndi_name: null(),
            __bindgen_anon_1: sdk::NDIlib_source_t__bindgen_ty_1 {
                p_url_address: null(),
            },
        },
        color_format: sdk::NDIlib_recv_color_format_fastest,
        bandwidth: sdk::NDIlib_recv_bandwidth_highest,
        allow_video_fields: false,
        p_ndi_recv_name: null(),
    };

    let instance = unsafe { sdk::NDIlib_recv_create_v3(&props) };

    if instance.is_null() {
        Err(CreateReceiveError::Failed)
    } else {
        Ok(ReceiveInstance {
            instance,
            frames: Mutex::new(HashMap::new()),
            frame_next_id: AtomicU64::new(0),
        })
    }
}
