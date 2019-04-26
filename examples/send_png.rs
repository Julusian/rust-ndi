extern crate ndi_sdk;
extern crate png;

use ndi_sdk::NDIInstance;
use std::fs::File;
use ndi_sdk::send::SendColorFormat;

fn main() {
    let instance: NDIInstance = ndi_sdk::load().expect("Failed to construct NDI instance");

    let decoder = png::Decoder::new(File::open("examples/sample-img.png").unwrap());
    let (info, mut reader) = decoder.read_info().unwrap();
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf).unwrap();

    // Create an NDI source that is called "My PNG" and is clocked to the video.
    let mut sender = instance
        .create_send_instance("My PNG".to_string(), false, false)
        .expect("Expected sender instance to be created");

    // We are going to create a frame
    let frame = ndi_sdk::send::create_ndi_send_video_frame(
        info.width as i32,
        info.height as i32,
        ndi_sdk::send::FrameFormatType::Progressive,
    )
    .with_data(buf, info.width as i32 * 4, SendColorFormat::Rgba)
    .build()
    .expect("Expected frame to be created");

    // We now submit the frame. Note that this call will be clocked so that we end up submitting at exactly 29.97fps.
    sender.send_video(frame);

    // Lets measure the performance for one minute
    println!("Source is now on output !");
    std::thread::sleep(std::time::Duration::from_secs(60))
}
