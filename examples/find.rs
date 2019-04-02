extern crate ndi_sdk;

use std::time::{Duration, Instant};

fn main() {
    let instance = ndi_sdk::load(None).expect("Failed to construct NDI instance");

    // Not required, but "correct" (see the SDK documentation.
    assert!(instance.init());

    {
        // We are going to create an NDI finder that locates sources on the network.
        let finder = instance.create_find_instance(true)
            .expect("Expected find instance to be created");

        let start = Instant::now();
        loop {
            // Run for one minute
            if start.elapsed() > Duration::from_secs(60) {
                break;
            }

            // Wait up till 5 seconds to check for new sources to be added or removed
            if !finder.wait_for_sources(5000) {
                println!("No change to the sources found.");
                continue;
            }

            // Get the updated list of sources
            let sources = finder.get_current_sources();

            // Display all the sources.
            println!("Network sources ({} found)", sources.len());
            for s in 0..sources.len() {
                println!("{}. {}", s, sources[s].name);
            }
        }
    }

    // Finished
    instance.destroy();
}
