use std::ffi::CStr;
use std::os::raw::c_char;
use std::str::Utf8Error;

fn c_char_ptr_to_str<'ptr_lifetime>(ptr: *const c_char) -> Result<&'ptr_lifetime str, Utf8Error> {
    unsafe { CStr::from_ptr(ptr) }.to_str()
}

pub(crate) fn try_c_char_ptr_to_str<'ptr_lifetime>(
    ptr: *const c_char,
) -> Result<Option<&'ptr_lifetime str>, Utf8Error> {
    if ptr.is_null() {
        Ok(None)
    } else {
        Ok(Some(c_char_ptr_to_str(ptr)?))
    }
}

#[cfg(test)]
mod tests {
    use super::try_c_char_ptr_to_str;

    use std::ffi::CString;
    use std::ptr;
    use std::str::Utf8Error;

    #[test]
    fn try_c_char_ptr_to_str_null_ptr() {
        assert_eq!(try_c_char_ptr_to_str(ptr::null()), Ok(None))
    }

    #[test]
    fn try_c_char_ptr_to_str_invalid_unicode() {
        let c_string = CString::new([0xC0, 0xC0, 0xC0]).expect("failed to create test CString");
        assert!(matches!(
            try_c_char_ptr_to_str(c_string.as_ptr()),
            Err(_Utf8Error)
        ))
    }

    #[test]
    fn try_c_char_ptr_to_str_all_valid() {
        let c_string = CString::new("A STRING HERE").expect("failed to create test CString");
        assert_eq!(
            try_c_char_ptr_to_str(c_string.as_ptr()),
            Ok(Some("A STRING HERE"))
        )
    }
}
