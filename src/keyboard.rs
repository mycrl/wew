/// Supported event bit flags.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventFlags {
    None = 0,
    CapsLockOn = 1 << 0,
    ShiftDown = 1 << 1,
    ControlDown = 1 << 2,
    AltDown = 1 << 3,
    LeftMouseButton = 1 << 4,
    MiddleMouseButton = 1 << 5,
    RightMouseButton = 1 << 6,
    CommandDown = 1 << 7,
    NumLockOn = 1 << 8,
    IsKeyPad = 1 << 9,
    IsLeft = 1 << 10,
    IsRight = 1 << 11,
    AltGrDown = 1 << 12,
    IsRepeat = 1 << 13,
    PrecisionScrollingDelta = 1 << 14,
    ScrollByPage = 1 << 15,
}

/// Represents modifier keys
///
/// This is mainly used for keyboard events
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Modifiers {
    Shift,
    Ctrl,
    Alt,
    Win,
}

/// Represents the type of key event
///
/// This is mainly used for keyboard events
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum KeyEventType {
    RawKeyDown,
    KeyDown,
    KeyUp,
    Char,
}

/// Represents a key event
///
/// This is mainly used for keyboard events
#[derive(Debug, Copy, Clone)]
pub struct KeyEvent {
    pub ty: KeyEventType,
    pub modifiers: EventFlags,
    pub windows_key_code: u32,
    pub native_key_code: u32,
    pub is_system_key: u32,
    pub character: u16,
    pub unmodified_character: u16,
    pub focus_on_editable_field: u32,
}

/// Get the state of the caps lock key
///
/// This function is used to get the state of the caps lock key.
///
/// # Returns
///
/// Returns true if the caps lock key is on, otherwise returns false.
pub fn get_capslock_state() -> bool {
    todo!()
}

pub struct KeyboardScanCodeAdapter {
    capslock_state: bool,
    event: KeyEvent,
}

impl Default for KeyboardScanCodeAdapter {
    fn default() -> Self {
        Self {
            event: unsafe { std::mem::zeroed() },
            capslock_state: get_capslock_state(),
        }
    }
}

impl KeyboardScanCodeAdapter {
    pub fn get_key_event(
        &mut self,
        code: u32,
        ty: KeyEventType,
        modifiers: EventFlags,
    ) -> &KeyEvent {
        self.event.ty = ty;
        self.event.modifiers = modifiers;
        self.event.native_key_code = code;

        if cfg!(target_os = "windows") {
            self.event.windows_key_code = code;
        }

        return &self.event;
    }
}
