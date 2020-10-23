extern crate libc;

use libc::{c_int, c_char};

extern "C" {
    pub fn OpenSSL_version(which: c_int) -> *const c_char;
}

#[cfg(test)]
mod tests {
    use std::ffi::{CStr, CString};
    use super::*;

    #[test]
    fn version_works() {
        unsafe {
            let name = CStr::from_ptr(OpenSSL_version(0));
            assert_eq!(name, &*CString::new("BoringSSL").unwrap());
        }
    }
}
