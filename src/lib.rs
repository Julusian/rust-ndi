#![feature(integer_atomics)]

#[allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    clippy::all
)]
mod sdk;

pub mod finder;
pub mod receive;
mod util;

pub fn init() -> bool {
    unsafe { sdk::NDIlib_initialize() }
}
pub fn destroy() {
    unsafe { sdk::NDIlib_destroy() }
}
