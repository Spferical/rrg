use std::ffi::CStr;

pub fn get_tsk_version() -> String {
    let cstr = unsafe { CStr::from_ptr(tsk_sys::tsk_version_get_str()) };
    String::from_utf8_lossy(cstr.to_bytes()).to_string()
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_get_tsk_version() {
        assert_eq!(get_tsk_version(), "4.12.1");
    }
}
