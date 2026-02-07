use core::ffi::c_int;
use std::ffi::CStr;

use bitflags::bitflags;
use cue_sdk_sys as ffi;

// ---------------------------------------------------------------------------
// PropertyId
// ---------------------------------------------------------------------------

/// Identifier for a device property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum PropertyId {
    PropertyArray = ffi::CorsairDevicePropertyId_CDPI_PropertyArray,
    MicEnabled = ffi::CorsairDevicePropertyId_CDPI_MicEnabled,
    SurroundSoundEnabled = ffi::CorsairDevicePropertyId_CDPI_SurroundSoundEnabled,
    SidetoneEnabled = ffi::CorsairDevicePropertyId_CDPI_SidetoneEnabled,
    EqualizerPreset = ffi::CorsairDevicePropertyId_CDPI_EqualizerPreset,
    PhysicalLayout = ffi::CorsairDevicePropertyId_CDPI_PhysicalLayout,
    LogicalLayout = ffi::CorsairDevicePropertyId_CDPI_LogicalLayout,
    MacroKeyArray = ffi::CorsairDevicePropertyId_CDPI_MacroKeyArray,
    BatteryLevel = ffi::CorsairDevicePropertyId_CDPI_BatteryLevel,
    ChannelLedCount = ffi::CorsairDevicePropertyId_CDPI_ChannelLedCount,
    ChannelDeviceCount = ffi::CorsairDevicePropertyId_CDPI_ChannelDeviceCount,
    ChannelDeviceLedCountArray = ffi::CorsairDevicePropertyId_CDPI_ChannelDeviceLedCountArray,
    ChannelDeviceTypeArray = ffi::CorsairDevicePropertyId_CDPI_ChannelDeviceTypeArray,
}

impl PropertyId {
    /// Convert to the FFI constant.
    pub(crate) fn to_ffi(self) -> ffi::CorsairDevicePropertyId {
        self as ffi::CorsairDevicePropertyId
    }
}

// ---------------------------------------------------------------------------
// PropertyFlags
// ---------------------------------------------------------------------------

bitflags! {
    /// Flags describing what operations are allowed on a property.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PropertyFlags: u32 {
        const CAN_READ  = ffi::CorsairPropertyFlag_CPF_CanRead;
        const CAN_WRITE = ffi::CorsairPropertyFlag_CPF_CanWrite;
        const INDEXED   = ffi::CorsairPropertyFlag_CPF_Indexed;
    }
}

// ---------------------------------------------------------------------------
// DataType
// ---------------------------------------------------------------------------

/// The data type of a property value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Boolean,
    Int32,
    Float64,
    String,
    BooleanArray,
    Int32Array,
    Float64Array,
    StringArray,
}

impl DataType {
    pub(crate) fn from_ffi(raw: ffi::CorsairDataType) -> Option<Self> {
        match raw {
            ffi::CorsairDataType_CT_Boolean => Some(Self::Boolean),
            ffi::CorsairDataType_CT_Int32 => Some(Self::Int32),
            ffi::CorsairDataType_CT_Float64 => Some(Self::Float64),
            ffi::CorsairDataType_CT_String => Some(Self::String),
            ffi::CorsairDataType_CT_Boolean_Array => Some(Self::BooleanArray),
            ffi::CorsairDataType_CT_Int32_Array => Some(Self::Int32Array),
            ffi::CorsairDataType_CT_Float64_Array => Some(Self::Float64Array),
            ffi::CorsairDataType_CT_String_Array => Some(Self::StringArray),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// PropertyInfo
// ---------------------------------------------------------------------------

/// Metadata about a device property (type and flags).
#[derive(Debug, Clone, Copy)]
pub struct PropertyInfo {
    pub data_type: DataType,
    pub flags: PropertyFlags,
}

// ---------------------------------------------------------------------------
// PropertyValue
// ---------------------------------------------------------------------------

/// An owned copy of a property value read from the SDK.
///
/// The SDK-allocated memory is freed immediately after the value is copied out,
/// so there are no dangling pointers.
#[derive(Debug, Clone)]
pub enum PropertyValue {
    Boolean(bool),
    Int32(i32),
    Float64(f64),
    String(std::string::String),
    BooleanArray(Vec<bool>),
    Int32Array(Vec<i32>),
    Float64Array(Vec<f64>),
    StringArray(Vec<std::string::String>),
}

impl PropertyValue {
    /// Extract an owned value from the raw FFI property, then free it.
    ///
    /// # Safety
    ///
    /// `prop` must point to a valid `CorsairProperty` returned by
    /// `CorsairReadDeviceProperty`, whose `value` union variant matches `type_`.
    /// After this call the SDK memory is freed via `CorsairFreeProperty`.
    pub(crate) unsafe fn from_ffi_and_free(prop: &mut ffi::CorsairProperty) -> Option<Self> {
        // In each arm below, we access the union variant that corresponds to
        // `prop.type_`.  This is safe because the caller guarantees that the
        // property was just initialised by the SDK with a matching type.
        let val = match prop.type_ {
            ffi::CorsairDataType_CT_Boolean => {
                // SAFETY: `type_` is `CT_Boolean`, so `value.boolean` is active.
                Some(PropertyValue::Boolean(unsafe { prop.value.boolean }))
            }
            ffi::CorsairDataType_CT_Int32 => {
                // SAFETY: `type_` is `CT_Int32`, so `value.int32` is active.
                Some(PropertyValue::Int32(unsafe { prop.value.int32 }))
            }
            ffi::CorsairDataType_CT_Float64 => {
                // SAFETY: `type_` is `CT_Float64`, so `value.float64` is active.
                Some(PropertyValue::Float64(unsafe { prop.value.float64 }))
            }
            ffi::CorsairDataType_CT_String => {
                // SAFETY: `type_` is `CT_String`, so `value.string` is active.
                let ptr = unsafe { prop.value.string };
                if ptr.is_null() {
                    Some(PropertyValue::String(std::string::String::new()))
                } else {
                    // SAFETY: The SDK returns a valid null-terminated C string.
                    let s = unsafe { CStr::from_ptr(ptr) }
                        .to_string_lossy()
                        .into_owned();
                    Some(PropertyValue::String(s))
                }
            }
            ffi::CorsairDataType_CT_Boolean_Array => {
                // SAFETY: `type_` is `CT_Boolean_Array`, so `value.boolean_array` is active.
                let arr = unsafe { prop.value.boolean_array };
                let slice = if arr.items.is_null() || arr.count == 0 {
                    &[]
                } else {
                    // SAFETY: The SDK guarantees `items` points to `count` valid bools.
                    unsafe { std::slice::from_raw_parts(arr.items, arr.count as usize) }
                };
                Some(PropertyValue::BooleanArray(slice.to_vec()))
            }
            ffi::CorsairDataType_CT_Int32_Array => {
                // SAFETY: `type_` is `CT_Int32_Array`, so `value.int32_array` is active.
                let arr = unsafe { prop.value.int32_array };
                let slice = if arr.items.is_null() || arr.count == 0 {
                    &[]
                } else {
                    // SAFETY: The SDK guarantees `items` points to `count` valid i32s.
                    unsafe { std::slice::from_raw_parts(arr.items, arr.count as usize) }
                };
                Some(PropertyValue::Int32Array(slice.to_vec()))
            }
            ffi::CorsairDataType_CT_Float64_Array => {
                // SAFETY: `type_` is `CT_Float64_Array`, so `value.float64_array` is active.
                let arr = unsafe { prop.value.float64_array };
                let slice = if arr.items.is_null() || arr.count == 0 {
                    &[]
                } else {
                    // SAFETY: The SDK guarantees `items` points to `count` valid f64s.
                    unsafe { std::slice::from_raw_parts(arr.items, arr.count as usize) }
                };
                Some(PropertyValue::Float64Array(slice.to_vec()))
            }
            ffi::CorsairDataType_CT_String_Array => {
                // SAFETY: `type_` is `CT_String_Array`, so `value.string_array` is active.
                let arr = unsafe { prop.value.string_array };
                let ptrs = if arr.items.is_null() || arr.count == 0 {
                    &[]
                } else {
                    // SAFETY: The SDK guarantees `items` points to `count` valid char pointers.
                    unsafe { std::slice::from_raw_parts(arr.items, arr.count as usize) }
                };
                let strings = ptrs
                    .iter()
                    .map(|&p| {
                        if p.is_null() {
                            std::string::String::new()
                        } else {
                            // SAFETY: Each non-null pointer is a valid C string from the SDK.
                            unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
                        }
                    })
                    .collect();
                Some(PropertyValue::StringArray(strings))
            }
            _ => None,
        };

        // SAFETY: `CorsairFreeProperty` releases SDK-allocated memory inside
        // the property.  It is safe to call on any property returned by
        // `CorsairReadDeviceProperty`, and must be called exactly once.
        unsafe {
            let _ = ffi::CorsairFreeProperty(prop as *mut ffi::CorsairProperty);
        }

        val
    }
}

/// Create a `CorsairProperty` for writing a boolean value.
pub(crate) fn make_bool_property(value: bool) -> ffi::CorsairProperty {
    // SAFETY: `CorsairProperty` is `#[repr(C)]` with no padding requirements
    // that zero-init would violate.  We immediately overwrite `type_` and `value`.
    let mut prop: ffi::CorsairProperty = unsafe { std::mem::zeroed() };
    prop.type_ = ffi::CorsairDataType_CT_Boolean;
    prop.value = ffi::CorsairDataValue { boolean: value };
    prop
}

/// Create a `CorsairProperty` for writing an i32 value.
pub(crate) fn make_int32_property(value: i32) -> ffi::CorsairProperty {
    // SAFETY: Same as `make_bool_property`.
    let mut prop: ffi::CorsairProperty = unsafe { std::mem::zeroed() };
    prop.type_ = ffi::CorsairDataType_CT_Int32;
    prop.value = ffi::CorsairDataValue {
        int32: value as c_int,
    };
    prop
}

/// Create a `CorsairProperty` for writing an f64 value.
pub(crate) fn make_float64_property(value: f64) -> ffi::CorsairProperty {
    // SAFETY: Same as `make_bool_property`.
    let mut prop: ffi::CorsairProperty = unsafe { std::mem::zeroed() };
    prop.type_ = ffi::CorsairDataType_CT_Float64;
    prop.value = ffi::CorsairDataValue { float64: value };
    prop
}
