use crate::finder::FindSource;
use crate::sdk;
use std::ffi::{CString, NulError};
use std::ptr::null;

// Messy return type to keep the CStrings alive long enough
pub fn to_ndi_source(
    source: &FindSource,
) -> Result<(CString, Option<CString>, sdk::NDIlib_source_t), NulError> {
    let source_name = CString::new(source.name.as_bytes())?;
    let source_url = match &source.url {
        None => None,
        Some(url) => Some(CString::new(url.as_bytes())?),
    };

    let res = sdk::NDIlib_source_t {
        p_ndi_name: source_name.as_ptr(),
        __bindgen_anon_1: sdk::NDIlib_source_t__bindgen_ty_1 {
            p_url_address: if let Some(url) = &source_url {
                url.as_ptr()
            } else {
                null()
            },
        },
    };

    Ok((source_name, source_url, res))
}
