use cue_sdk_sys as ffi;
use num_traits::FromPrimitive;

/// The layout of the given device, with keyboards having a `physical_layout` and `logical_layout`
/// while mice only have a `physical_layout`.
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceLayout {
    Keyboard {
        physical_layout: PhysicalLayout,
        logical_layout: LogicalLayout,
    },
    Mouse {
        physical_layout: PhysicalLayout,
    },
}

impl DeviceLayout {
    pub(crate) fn from_ffi_values(
        physical_layout: ffi::CorsairPhysicalLayout,
        logical_layout: ffi::CorsairLogicalLayout,
    ) -> Option<Self> {
        let physical = PhysicalLayout::from_u32(physical_layout);
        let logical = LogicalLayout::from_u32(logical_layout);

        match physical {
            None => None,
            Some(pl) => match logical {
                Some(ll) => Some(DeviceLayout::Keyboard {
                    physical_layout: pl,
                    logical_layout: ll,
                }),
                None => Some(DeviceLayout::Mouse {
                    physical_layout: pl,
                }),
            },
        }
    }
}

/// The various physical layouts that a keyboard `CueDevice` can have.
#[derive(Debug, Copy, Clone, FromPrimitive, PartialEq)]
#[repr(u32)]
pub enum PhysicalLayout {
    KeyboardUs = ffi::CorsairPhysicalLayout_CPL_US,
    KeyboardUk = ffi::CorsairPhysicalLayout_CPL_UK,
    KeyboardBr = ffi::CorsairPhysicalLayout_CPL_BR,
    KeyboardJp = ffi::CorsairPhysicalLayout_CPL_JP,
    KeyboardKr = ffi::CorsairPhysicalLayout_CPL_KR,
    MouseLedCount1 = ffi::CorsairPhysicalLayout_CPL_Zones1,
    MouseLedCount2 = ffi::CorsairPhysicalLayout_CPL_Zones2,
    MouseLedCount3 = ffi::CorsairPhysicalLayout_CPL_Zones3,
    MouseLedCount4 = ffi::CorsairPhysicalLayout_CPL_Zones4,
}

/// The various logical layouts a mouse `CueDevice` can have.
#[derive(Debug, Copy, Clone, FromPrimitive, PartialEq)]
#[repr(u32)]
pub enum LogicalLayout {
    KeyboardUsInt = ffi::CorsairLogicalLayout_CLL_US_Int,
    KeyboardNa = ffi::CorsairLogicalLayout_CLL_NA,
    KeyboardEu = ffi::CorsairLogicalLayout_CLL_EU,
    KeyboardUk = ffi::CorsairLogicalLayout_CLL_UK,
    KeyboardBe = ffi::CorsairLogicalLayout_CLL_BE,
    KeyboardBr = ffi::CorsairLogicalLayout_CLL_BR,
    KeyboardCh = ffi::CorsairLogicalLayout_CLL_CH,
    KeyboardCn = ffi::CorsairLogicalLayout_CLL_CN,
    KeyboardDe = ffi::CorsairLogicalLayout_CLL_DE,
    KeyboardEs = ffi::CorsairLogicalLayout_CLL_ES,
    KeyboardFr = ffi::CorsairLogicalLayout_CLL_FR,
    KeyboardIt = ffi::CorsairLogicalLayout_CLL_IT,
    KeyboardNd = ffi::CorsairLogicalLayout_CLL_ND,
    KeyboardRu = ffi::CorsairLogicalLayout_CLL_RU,
    KeyboardJp = ffi::CorsairLogicalLayout_CLL_JP,
    KeyboardKr = ffi::CorsairLogicalLayout_CLL_KR,
    KeyboardTw = ffi::CorsairLogicalLayout_CLL_TW,
    KeyboardMex = ffi::CorsairLogicalLayout_CLL_MEX,
}

#[cfg(test)]
mod tests {

    use super::{DeviceLayout, LogicalLayout, PhysicalLayout};
    use cue_sdk_sys as ffi;

    #[test]
    fn from_ffi_values_no_physical() {
        let layout = DeviceLayout::from_ffi_values(
            ffi::CorsairPhysicalLayout_CPL_Invalid,
            ffi::CorsairLogicalLayout_CLL_DE,
        );
        assert_eq!(layout, None)
    }

    #[test]
    fn from_ffi_values_physical_but_no_logical() {
        let layout = DeviceLayout::from_ffi_values(
            ffi::CorsairPhysicalLayout_CPL_Zones2,
            ffi::CorsairLogicalLayout_CLL_Invalid,
        );
        assert_eq!(
            layout,
            Some(DeviceLayout::Mouse {
                physical_layout: PhysicalLayout::MouseLedCount2
            })
        )
    }

    #[test]
    fn from_ffi_values_both() {
        let layout = DeviceLayout::from_ffi_values(
            ffi::CorsairPhysicalLayout_CPL_US,
            ffi::CorsairLogicalLayout_CLL_US_Int,
        );
        assert_eq!(
            layout,
            Some(DeviceLayout::Keyboard {
                physical_layout: PhysicalLayout::KeyboardUs,
                logical_layout: LogicalLayout::KeyboardUsInt
            })
        )
    }
}
