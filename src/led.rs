use cue_sdk_sys as ffi;

// ---------------------------------------------------------------------------
// LedColor
// ---------------------------------------------------------------------------

/// An LED color value that is layout-compatible with [`ffi::CorsairLedColor`].
///
/// The `#[repr(C)]` layout means a `&[LedColor]` slice can be passed directly
/// to FFI functions via pointer cast — no per-element copying required.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LedColor {
    /// LED locally-unique identifier.
    pub id: u32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl LedColor {
    /// Create a new LED color.
    pub fn new(id: u32, r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { id, r, g, b, a }
    }

    /// Create an opaque RGB color (alpha = 255).
    pub fn rgb(id: u32, r: u8, g: u8, b: u8) -> Self {
        Self::new(id, r, g, b, 255)
    }
}

// Safety: LedColor is a plain-old-data type with the exact same layout as
// CorsairLedColor.  These assertions prove it at compile time.
const _: () = {
    assert!(
        std::mem::size_of::<LedColor>() == std::mem::size_of::<ffi::CorsairLedColor>(),
        "LedColor size mismatch"
    );
    assert!(
        std::mem::align_of::<LedColor>() == std::mem::align_of::<ffi::CorsairLedColor>(),
        "LedColor alignment mismatch"
    );
};

// ---------------------------------------------------------------------------
// LedPosition
// ---------------------------------------------------------------------------

/// Position of an LED on a device, as reported by the SDK.
#[derive(Debug, Clone, Copy)]
pub struct LedPosition {
    /// LED locally-unique identifier.
    pub id: u32,
    /// X coordinate (center).
    pub cx: f64,
    /// Y coordinate (center).
    pub cy: f64,
}

impl LedPosition {
    pub(crate) fn from_ffi(raw: &ffi::CorsairLedPosition) -> Self {
        Self {
            id: raw.id,
            cx: raw.cx,
            cy: raw.cy,
        }
    }
}

// ---------------------------------------------------------------------------
// Keyboard LED constants
// ---------------------------------------------------------------------------

/// Named constants for keyboard LED LUIDs from the `CLG_Keyboard` group.
///
/// These correspond to `CorsairLedId_Keyboard_CLK_*` values and can be used
/// as the `id` field in [`LedColor`].
pub mod keyboard {
    use cue_sdk_sys as ffi;

    pub const ESCAPE: u32 = ffi::CorsairLedId_Keyboard_CLK_Escape;
    pub const F1: u32 = ffi::CorsairLedId_Keyboard_CLK_F1;
    pub const F2: u32 = ffi::CorsairLedId_Keyboard_CLK_F2;
    pub const F3: u32 = ffi::CorsairLedId_Keyboard_CLK_F3;
    pub const F4: u32 = ffi::CorsairLedId_Keyboard_CLK_F4;
    pub const F5: u32 = ffi::CorsairLedId_Keyboard_CLK_F5;
    pub const F6: u32 = ffi::CorsairLedId_Keyboard_CLK_F6;
    pub const F7: u32 = ffi::CorsairLedId_Keyboard_CLK_F7;
    pub const F8: u32 = ffi::CorsairLedId_Keyboard_CLK_F8;
    pub const F9: u32 = ffi::CorsairLedId_Keyboard_CLK_F9;
    pub const F10: u32 = ffi::CorsairLedId_Keyboard_CLK_F10;
    pub const F11: u32 = ffi::CorsairLedId_Keyboard_CLK_F11;
    pub const F12: u32 = ffi::CorsairLedId_Keyboard_CLK_F12;
    pub const GRAVE_ACCENT: u32 = ffi::CorsairLedId_Keyboard_CLK_GraveAccentAndTilde;
    pub const KEY_1: u32 = ffi::CorsairLedId_Keyboard_CLK_1;
    pub const KEY_2: u32 = ffi::CorsairLedId_Keyboard_CLK_2;
    pub const KEY_3: u32 = ffi::CorsairLedId_Keyboard_CLK_3;
    pub const KEY_4: u32 = ffi::CorsairLedId_Keyboard_CLK_4;
    pub const KEY_5: u32 = ffi::CorsairLedId_Keyboard_CLK_5;
    pub const KEY_6: u32 = ffi::CorsairLedId_Keyboard_CLK_6;
    pub const KEY_7: u32 = ffi::CorsairLedId_Keyboard_CLK_7;
    pub const KEY_8: u32 = ffi::CorsairLedId_Keyboard_CLK_8;
    pub const KEY_9: u32 = ffi::CorsairLedId_Keyboard_CLK_9;
    pub const KEY_0: u32 = ffi::CorsairLedId_Keyboard_CLK_0;
    pub const MINUS: u32 = ffi::CorsairLedId_Keyboard_CLK_MinusAndUnderscore;
    pub const EQUALS: u32 = ffi::CorsairLedId_Keyboard_CLK_EqualsAndPlus;
    pub const BACKSPACE: u32 = ffi::CorsairLedId_Keyboard_CLK_Backspace;
    pub const TAB: u32 = ffi::CorsairLedId_Keyboard_CLK_Tab;
    pub const Q: u32 = ffi::CorsairLedId_Keyboard_CLK_Q;
    pub const W: u32 = ffi::CorsairLedId_Keyboard_CLK_W;
    pub const E: u32 = ffi::CorsairLedId_Keyboard_CLK_E;
    pub const R: u32 = ffi::CorsairLedId_Keyboard_CLK_R;
    pub const T: u32 = ffi::CorsairLedId_Keyboard_CLK_T;
    pub const Y: u32 = ffi::CorsairLedId_Keyboard_CLK_Y;
    pub const U: u32 = ffi::CorsairLedId_Keyboard_CLK_U;
    pub const I: u32 = ffi::CorsairLedId_Keyboard_CLK_I;
    pub const O: u32 = ffi::CorsairLedId_Keyboard_CLK_O;
    pub const P: u32 = ffi::CorsairLedId_Keyboard_CLK_P;
    pub const BRACKET_LEFT: u32 = ffi::CorsairLedId_Keyboard_CLK_BracketLeft;
    pub const BRACKET_RIGHT: u32 = ffi::CorsairLedId_Keyboard_CLK_BracketRight;
    pub const CAPS_LOCK: u32 = ffi::CorsairLedId_Keyboard_CLK_CapsLock;
    pub const A: u32 = ffi::CorsairLedId_Keyboard_CLK_A;
    pub const S: u32 = ffi::CorsairLedId_Keyboard_CLK_S;
    pub const D: u32 = ffi::CorsairLedId_Keyboard_CLK_D;
    pub const F: u32 = ffi::CorsairLedId_Keyboard_CLK_F;
    pub const G: u32 = ffi::CorsairLedId_Keyboard_CLK_G;
    pub const H: u32 = ffi::CorsairLedId_Keyboard_CLK_H;
    pub const J: u32 = ffi::CorsairLedId_Keyboard_CLK_J;
    pub const K: u32 = ffi::CorsairLedId_Keyboard_CLK_K;
    pub const L: u32 = ffi::CorsairLedId_Keyboard_CLK_L;
    pub const SEMICOLON: u32 = ffi::CorsairLedId_Keyboard_CLK_SemicolonAndColon;
    pub const APOSTROPHE: u32 = ffi::CorsairLedId_Keyboard_CLK_ApostropheAndDoubleQuote;
    pub const BACKSLASH: u32 = ffi::CorsairLedId_Keyboard_CLK_Backslash;
    pub const ENTER: u32 = ffi::CorsairLedId_Keyboard_CLK_Enter;
    pub const LEFT_SHIFT: u32 = ffi::CorsairLedId_Keyboard_CLK_LeftShift;
    pub const NON_US_BACKSLASH: u32 = ffi::CorsairLedId_Keyboard_CLK_NonUsBackslash;
    pub const Z: u32 = ffi::CorsairLedId_Keyboard_CLK_Z;
    pub const X: u32 = ffi::CorsairLedId_Keyboard_CLK_X;
    pub const C: u32 = ffi::CorsairLedId_Keyboard_CLK_C;
    pub const V: u32 = ffi::CorsairLedId_Keyboard_CLK_V;
    pub const B: u32 = ffi::CorsairLedId_Keyboard_CLK_B;
    pub const N: u32 = ffi::CorsairLedId_Keyboard_CLK_N;
    pub const M: u32 = ffi::CorsairLedId_Keyboard_CLK_M;
    pub const COMMA: u32 = ffi::CorsairLedId_Keyboard_CLK_CommaAndLessThan;
    pub const PERIOD: u32 = ffi::CorsairLedId_Keyboard_CLK_PeriodAndBiggerThan;
    pub const SLASH: u32 = ffi::CorsairLedId_Keyboard_CLK_SlashAndQuestionMark;
    pub const RIGHT_SHIFT: u32 = ffi::CorsairLedId_Keyboard_CLK_RightShift;
    pub const LEFT_CTRL: u32 = ffi::CorsairLedId_Keyboard_CLK_LeftCtrl;
    pub const LEFT_GUI: u32 = ffi::CorsairLedId_Keyboard_CLK_LeftGui;
    pub const LEFT_ALT: u32 = ffi::CorsairLedId_Keyboard_CLK_LeftAlt;
    pub const SPACE: u32 = ffi::CorsairLedId_Keyboard_CLK_Space;
    pub const RIGHT_ALT: u32 = ffi::CorsairLedId_Keyboard_CLK_RightAlt;
    pub const RIGHT_GUI: u32 = ffi::CorsairLedId_Keyboard_CLK_RightGui;
    pub const APPLICATION: u32 = ffi::CorsairLedId_Keyboard_CLK_Application;
    pub const RIGHT_CTRL: u32 = ffi::CorsairLedId_Keyboard_CLK_RightCtrl;
    pub const LED_PROGRAMMING: u32 = ffi::CorsairLedId_Keyboard_CLK_LedProgramming;
    pub const LANG1: u32 = ffi::CorsairLedId_Keyboard_CLK_Lang1;
    pub const LANG2: u32 = ffi::CorsairLedId_Keyboard_CLK_Lang2;
    pub const INTERNATIONAL1: u32 = ffi::CorsairLedId_Keyboard_CLK_International1;
    pub const INTERNATIONAL2: u32 = ffi::CorsairLedId_Keyboard_CLK_International2;
    pub const INTERNATIONAL3: u32 = ffi::CorsairLedId_Keyboard_CLK_International3;
    pub const INTERNATIONAL4: u32 = ffi::CorsairLedId_Keyboard_CLK_International4;
    pub const INTERNATIONAL5: u32 = ffi::CorsairLedId_Keyboard_CLK_International5;
    pub const PRINT_SCREEN: u32 = ffi::CorsairLedId_Keyboard_CLK_PrintScreen;
    pub const SCROLL_LOCK: u32 = ffi::CorsairLedId_Keyboard_CLK_ScrollLock;
    pub const PAUSE_BREAK: u32 = ffi::CorsairLedId_Keyboard_CLK_PauseBreak;
    pub const INSERT: u32 = ffi::CorsairLedId_Keyboard_CLK_Insert;
    pub const HOME: u32 = ffi::CorsairLedId_Keyboard_CLK_Home;
    pub const PAGE_UP: u32 = ffi::CorsairLedId_Keyboard_CLK_PageUp;
    pub const DELETE: u32 = ffi::CorsairLedId_Keyboard_CLK_Delete;
    pub const END: u32 = ffi::CorsairLedId_Keyboard_CLK_End;
    pub const PAGE_DOWN: u32 = ffi::CorsairLedId_Keyboard_CLK_PageDown;
    pub const UP_ARROW: u32 = ffi::CorsairLedId_Keyboard_CLK_UpArrow;
    pub const LEFT_ARROW: u32 = ffi::CorsairLedId_Keyboard_CLK_LeftArrow;
    pub const DOWN_ARROW: u32 = ffi::CorsairLedId_Keyboard_CLK_DownArrow;
    pub const RIGHT_ARROW: u32 = ffi::CorsairLedId_Keyboard_CLK_RightArrow;
    pub const NON_US_TILDE: u32 = ffi::CorsairLedId_Keyboard_CLK_NonUsTilde;
    pub const BRIGHTNESS: u32 = ffi::CorsairLedId_Keyboard_CLK_Brightness;
    pub const WIN_LOCK: u32 = ffi::CorsairLedId_Keyboard_CLK_WinLock;
    pub const MUTE: u32 = ffi::CorsairLedId_Keyboard_CLK_Mute;
    pub const STOP: u32 = ffi::CorsairLedId_Keyboard_CLK_Stop;
    pub const SCAN_PREVIOUS_TRACK: u32 = ffi::CorsairLedId_Keyboard_CLK_ScanPreviousTrack;
    pub const PLAY_PAUSE: u32 = ffi::CorsairLedId_Keyboard_CLK_PlayPause;
    pub const SCAN_NEXT_TRACK: u32 = ffi::CorsairLedId_Keyboard_CLK_ScanNextTrack;
    pub const NUM_LOCK: u32 = ffi::CorsairLedId_Keyboard_CLK_NumLock;
    pub const KEYPAD_SLASH: u32 = ffi::CorsairLedId_Keyboard_CLK_KeypadSlash;
    pub const KEYPAD_ASTERISK: u32 = ffi::CorsairLedId_Keyboard_CLK_KeypadAsterisk;
    pub const KEYPAD_MINUS: u32 = ffi::CorsairLedId_Keyboard_CLK_KeypadMinus;
    pub const KEYPAD_7: u32 = ffi::CorsairLedId_Keyboard_CLK_Keypad7;
    pub const KEYPAD_8: u32 = ffi::CorsairLedId_Keyboard_CLK_Keypad8;
    pub const KEYPAD_9: u32 = ffi::CorsairLedId_Keyboard_CLK_Keypad9;
    pub const KEYPAD_PLUS: u32 = ffi::CorsairLedId_Keyboard_CLK_KeypadPlus;
    pub const KEYPAD_4: u32 = ffi::CorsairLedId_Keyboard_CLK_Keypad4;
    pub const KEYPAD_5: u32 = ffi::CorsairLedId_Keyboard_CLK_Keypad5;
    pub const KEYPAD_6: u32 = ffi::CorsairLedId_Keyboard_CLK_Keypad6;
    pub const KEYPAD_1: u32 = ffi::CorsairLedId_Keyboard_CLK_Keypad1;
    pub const KEYPAD_2: u32 = ffi::CorsairLedId_Keyboard_CLK_Keypad2;
    pub const KEYPAD_3: u32 = ffi::CorsairLedId_Keyboard_CLK_Keypad3;
    pub const KEYPAD_COMMA: u32 = ffi::CorsairLedId_Keyboard_CLK_KeypadComma;
    pub const KEYPAD_ENTER: u32 = ffi::CorsairLedId_Keyboard_CLK_KeypadEnter;
    pub const KEYPAD_0: u32 = ffi::CorsairLedId_Keyboard_CLK_Keypad0;
    pub const KEYPAD_PERIOD_DELETE: u32 = ffi::CorsairLedId_Keyboard_CLK_KeypadPeriodAndDelete;
    pub const VOLUME_UP: u32 = ffi::CorsairLedId_Keyboard_CLK_VolumeUp;
    pub const VOLUME_DOWN: u32 = ffi::CorsairLedId_Keyboard_CLK_VolumeDown;
    pub const MR: u32 = ffi::CorsairLedId_Keyboard_CLK_MR;
    pub const M1: u32 = ffi::CorsairLedId_Keyboard_CLK_M1;
    pub const M2: u32 = ffi::CorsairLedId_Keyboard_CLK_M2;
    pub const M3: u32 = ffi::CorsairLedId_Keyboard_CLK_M3;
    pub const FN: u32 = ffi::CorsairLedId_Keyboard_CLK_Fn;
}
