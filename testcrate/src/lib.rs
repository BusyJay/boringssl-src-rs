extern crate libc;

use libc::{c_char, c_int};

extern "C" {
    pub fn OpenSSL_version(which: c_int) -> *const c_char;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};

    #[test]
    fn version_works() {
        unsafe {
            let name = CStr::from_ptr(OpenSSL_version(0));
            assert_eq!(name, &*CString::new("BoringSSL").unwrap());
        }
    }
}
