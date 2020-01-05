extern crate ndi_sdk;

use ndi_sdk::receive::{ReceiveBandwidth, ReceiveCaptureResult, ReceiveColorFormat, ReceiveInstanceExt};
use ndi_sdk::NDIInstance;
use std::time::{Duration, Instant};

fn main() {
    let instance: NDIInstance = ndi_sdk::load().expect("Failed to construct NDI instance");

    let source = {
        // Create a finder
        let finder = instance
            .create_find_instance(true)
            .expect("Expected find instance to be created");

        // Wait until there is one source
        loop {
            println!("Looking for sources ...");
            finder.wait_for_sources(1000);
            let sources = finder.get_current_sources();
            if sources.len() > 0 {
                break sources[0].clone();
            }
        }
    };

    println!("Found source: {}", source.name);

    // We now have at least one source, so we create a receiver to look at it.
    let receiver = instance
        .create_receive_instance(ReceiveBandwidth::Highest, ReceiveColorFormat::Fastest)
        .expect("create receiver");

    // Connect to our sources
    assert!(receiver.connect(Some(&source)));

    let start = Instant::now();
    loop {
        // Run for five minute
        if start.elapsed() > Duration::from_secs(5 * 60) {
            break;
        }

        let c = receiver.receive_capture(true, true, false, 5000);
        match c {
            Err(e) => println!("Capture failed: {:?}", e),
            Ok(c) => match c {
                ReceiveCaptureResult::None => println!("No data received."),
                ReceiveCaptureResult::Video(video) => {
                    println!("Video data received ({}x{}).", video.width, video.height);
                    if let Some(data) = video.lock_data() {
                        println!("  Got {} bytes", data.len());
                    }
                }
                ReceiveCaptureResult::Audio(audio) => {
                    println!("Audio data received ({} samples).", audio.sample_count);
                    if let Some(data) = audio.lock_data() {
                        println!("  Got {} bytes", data.len());
                    }
                }
                _ => {}
            },
        }
    }
}
