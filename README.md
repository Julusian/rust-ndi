# Rust NDI SDK

NDI SDK wrapper for use in Rust.
This aims to be an easy to use and safe wrapper around the official C SDK.

Note: This is very incomplete, but is working. The examples currently follow the official examples as closely as possible

## Installation

To use the library, you need the so files from the official SDK.
The library will search a few locations for the so files. It will try the local directory, the environment variable
NDI_RUNTIME_DIR_V5 (as recommended by the sdk), then the system default search paths.

For the examples, placing the so files into a lib folder in the repository will cause them to be found and used.

## Usage

See the examples for more information.

The library can be linked as a dependency, or dynamically. The same api is exposed in both cases for simplicity.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.