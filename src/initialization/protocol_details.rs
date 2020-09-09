use cue_sdk_sys as ffi;

use semver::{SemVerError, Version};
use std::str::Utf8Error;

use crate::internal::try_c_char_ptr_to_str;

#[derive(Debug, Clone, PartialEq)]
pub struct ProtocolDetails {
    pub sdk_version: Version,
    pub server_version: Option<Version>,
    pub sdk_protocol_version: i32,
    pub server_protocol_version: i32,
    pub breaking_changes: bool,
}

#[derive(Debug, Clone, Fail, PartialEq)]
pub enum ProtocolDetailsFromFfiError {
    #[fail(
        display = "Failed to parse string on field: {}, with error: {}",
        field, error
    )]
    StringParsingError {
        field: String,
        #[cause]
        error: Utf8Error,
    },
    #[fail(
        display = "Failed to parse version on field: {}, with error: {}",
        field, error
    )]
    VersionParsingError {
        field: String,
        #[cause]
        error: SemVerError,
    },
    #[fail(display = "Unexpected null pointer on field: {}", _0)]
    NullPtrError(String),
}

fn parse_version(field_name: &str, str: &str) -> Result<Version, ProtocolDetailsFromFfiError> {
    Version::parse(str).map_err(|e| ProtocolDetailsFromFfiError::VersionParsingError {
        field: field_name.to_owned(),
        error: e,
    })
}

impl ProtocolDetails {
    pub(crate) fn from_ffi(
        details: ffi::CorsairProtocolDetails,
    ) -> Result<Self, ProtocolDetailsFromFfiError> {
        let sdk_version_str = try_c_char_ptr_to_str(details.sdkVersion)
            .map_err(|e| ProtocolDetailsFromFfiError::StringParsingError {
                field: "sdk_version".to_owned(),
                error: e,
            })?
            .ok_or_else(|| ProtocolDetailsFromFfiError::NullPtrError("sdk_version".to_owned()))?;

        let sdk_version = parse_version("sdk_version", sdk_version_str)?;

        let server_version_str = try_c_char_ptr_to_str(details.serverVersion).map_err(|e| {
            ProtocolDetailsFromFfiError::StringParsingError {
                field: "server_version".to_owned(),
                error: e,
            }
        })?;

        let server_version = match server_version_str {
            Some(vs) => Some(parse_version("server_version", vs)?),
            None => None,
        };

        Ok(ProtocolDetails {
            sdk_version,
            server_version,
            sdk_protocol_version: details.sdkProtocolVersion,
            server_protocol_version: details.serverProtocolVersion,
            breaking_changes: details.breakingChanges,
        })
    }
}

#[cfg(test)]
mod tests {
    use cue_sdk_sys as ffi;
    use semver::Version;

    use std::ffi::CString;
    use std::ptr;

    use super::ProtocolDetails;
    use crate::initialization::ProtocolDetailsFromFfiError;

    #[test]
    fn pd_from_ffi_sdk_version_null_ptr() {
        let server_version = CString::new("4.1.0").unwrap();

        let ffi_details = ffi::CorsairProtocolDetails {
            sdkVersion: ptr::null(),
            serverVersion: server_version.as_ptr(),
            sdkProtocolVersion: 3,
            serverProtocolVersion: 2,
            breakingChanges: true,
        };

        assert!(
            matches!(ProtocolDetails::from_ffi(ffi_details).unwrap_err(),
            ProtocolDetailsFromFfiError::NullPtrError(field) if field == "sdk_version")
        );
    }

    #[test]
    fn pd_from_ffi_invalid_sdk_version_utf8() {
        let server_version = CString::new("4.1.0").unwrap();
        let sdk_version = CString::new([0xC0, 0x30]).unwrap();

        let ffi_details = ffi::CorsairProtocolDetails {
            sdkVersion: sdk_version.as_ptr(),
            serverVersion: server_version.as_ptr(),
            sdkProtocolVersion: 3,
            serverProtocolVersion: 2,
            breakingChanges: true,
        };

        assert!(
            matches!(ProtocolDetails::from_ffi(ffi_details).unwrap_err(),
            ProtocolDetailsFromFfiError::StringParsingError {field, ..} if field == "sdk_version")
        );
    }

    #[test]
    fn pd_from_ffi_invalid_sdk_version_value() {
        let server_version = CString::new("3.5.0").unwrap();
        let sdk_version = CString::new("Hello!").unwrap();

        let ffi_details = ffi::CorsairProtocolDetails {
            sdkVersion: sdk_version.as_ptr(),
            serverVersion: server_version.as_ptr(),
            sdkProtocolVersion: 3,
            serverProtocolVersion: 2,
            breakingChanges: true,
        };

        assert!(
            matches!(ProtocolDetails::from_ffi(ffi_details).unwrap_err(),
            ProtocolDetailsFromFfiError::VersionParsingError {field, .. } if field == "sdk_version")
        );
    }

    #[test]
    fn pd_from_ffi_invalid_server_version_utf8() {
        let server_version = CString::new([0xC0, 0x30]).unwrap();
        let sdk_version = CString::new("2.4.12").unwrap();

        let ffi_details = ffi::CorsairProtocolDetails {
            sdkVersion: sdk_version.as_ptr(),
            serverVersion: server_version.as_ptr(),
            sdkProtocolVersion: 3,
            serverProtocolVersion: 2,
            breakingChanges: true,
        };

        assert!(
            matches!(ProtocolDetails::from_ffi(ffi_details).unwrap_err(),
            ProtocolDetailsFromFfiError::StringParsingError {field, ..} if field == "server_version")
        );
    }

    #[test]
    fn pd_from_ffi_invalid_server_version_value() {
        let server_version = CString::new("35.0").unwrap();
        let sdk_version = CString::new("2.4.12").unwrap();

        let ffi_details = ffi::CorsairProtocolDetails {
            sdkVersion: sdk_version.as_ptr(),
            serverVersion: server_version.as_ptr(),
            sdkProtocolVersion: 3,
            serverProtocolVersion: 2,
            breakingChanges: true,
        };

        assert!(
            matches!(ProtocolDetails::from_ffi(ffi_details).unwrap_err(),
            ProtocolDetailsFromFfiError::VersionParsingError{ field, .. } if field == "server_version")
        );
    }

    #[test]
    fn pd_from_ffi_ok_with_missing_server_version() {
        let sdk_version = CString::new("5.1.1").unwrap();

        let ffi_details = ffi::CorsairProtocolDetails {
            sdkVersion: sdk_version.as_ptr(),
            serverVersion: ptr::null(),
            sdkProtocolVersion: 5,
            serverProtocolVersion: 4,
            breakingChanges: false,
        };

        assert_eq!(
            ProtocolDetails::from_ffi(ffi_details).unwrap(),
            ProtocolDetails {
                sdk_version: Version::new(5, 1, 1),
                server_version: None,
                sdk_protocol_version: 5,
                server_protocol_version: 4,
                breaking_changes: false,
            }
        )
    }

    #[test]
    fn pd_from_ffi_ok_all_valid() {
        let server_version = CString::new("3.5.0").unwrap();
        let sdk_version = CString::new("2.4.12").unwrap();

        let ffi_details = ffi::CorsairProtocolDetails {
            sdkVersion: sdk_version.as_ptr(),
            serverVersion: server_version.as_ptr(),
            sdkProtocolVersion: 3,
            serverProtocolVersion: 2,
            breakingChanges: true,
        };

        assert_eq!(
            ProtocolDetails::from_ffi(ffi_details).unwrap(),
            ProtocolDetails {
                sdk_version: Version::new(2, 4, 12),
                server_version: Some(Version::new(3, 5, 0)),
                sdk_protocol_version: 3,
                server_protocol_version: 2,
                breaking_changes: true,
            }
        )
    }
}
