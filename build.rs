// build.rs

fn main() {
    //    println!("cargo:rustc-link-search=native=input");

    if cfg!(not(feature = "dynamic-link")) {
        // Static link against it
        println!("cargo:rustc-link-lib=ndi");
    }
}
