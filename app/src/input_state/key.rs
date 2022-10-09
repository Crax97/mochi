use strum::EnumCount;
use strum_macros::EnumCount;
use winit::event::ModifiersState;

#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, EnumCount)]
pub enum Key {
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Key0,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    Escape,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    Snapshot,
    Scroll,
    Pause,

    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    Left,
    Up,
    Right,
    Down,

    Backspace,
    Return,
    Space,

    Compose,

    Caret,

    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadDecimal,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,

    AbntC1,
    AbntC2,
    Apostrophe,
    Apps,
    Asterisk,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Mute,
    MyComputer,
    NavigateForward,
    NavigateBackward,
    NextTrack,
    NoConvert,
    OEM102,
    Period,
    PlayPause,
    Plus,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}

impl From<winit::event::VirtualKeyCode> for Key {
    fn from(vk: winit::event::VirtualKeyCode) -> Self {
        match vk {
            winit::event::VirtualKeyCode::Key1 => Key::Key1,
            winit::event::VirtualKeyCode::Key2 => Key::Key2,
            winit::event::VirtualKeyCode::Key3 => Key::Key3,
            winit::event::VirtualKeyCode::Key4 => Key::Key4,
            winit::event::VirtualKeyCode::Key5 => Key::Key5,
            winit::event::VirtualKeyCode::Key6 => Key::Key6,
            winit::event::VirtualKeyCode::Key7 => Key::Key7,
            winit::event::VirtualKeyCode::Key8 => Key::Key8,
            winit::event::VirtualKeyCode::Key9 => Key::Key9,
            winit::event::VirtualKeyCode::Key0 => Key::Key0,
            winit::event::VirtualKeyCode::A => Key::A,
            winit::event::VirtualKeyCode::B => Key::B,
            winit::event::VirtualKeyCode::C => Key::C,
            winit::event::VirtualKeyCode::D => Key::D,
            winit::event::VirtualKeyCode::E => Key::E,
            winit::event::VirtualKeyCode::F => Key::F,
            winit::event::VirtualKeyCode::G => Key::G,
            winit::event::VirtualKeyCode::H => Key::H,
            winit::event::VirtualKeyCode::I => Key::I,
            winit::event::VirtualKeyCode::J => Key::J,
            winit::event::VirtualKeyCode::K => Key::K,
            winit::event::VirtualKeyCode::L => Key::L,
            winit::event::VirtualKeyCode::M => Key::M,
            winit::event::VirtualKeyCode::N => Key::N,
            winit::event::VirtualKeyCode::O => Key::O,
            winit::event::VirtualKeyCode::P => Key::P,
            winit::event::VirtualKeyCode::Q => Key::Q,
            winit::event::VirtualKeyCode::R => Key::R,
            winit::event::VirtualKeyCode::S => Key::S,
            winit::event::VirtualKeyCode::T => Key::T,
            winit::event::VirtualKeyCode::U => Key::U,
            winit::event::VirtualKeyCode::V => Key::V,
            winit::event::VirtualKeyCode::W => Key::W,
            winit::event::VirtualKeyCode::X => Key::X,
            winit::event::VirtualKeyCode::Y => Key::Y,
            winit::event::VirtualKeyCode::Z => Key::Z,
            winit::event::VirtualKeyCode::Escape => Key::Escape,
            winit::event::VirtualKeyCode::F1 => Key::F1,
            winit::event::VirtualKeyCode::F2 => Key::F2,
            winit::event::VirtualKeyCode::F3 => Key::F3,
            winit::event::VirtualKeyCode::F4 => Key::F4,
            winit::event::VirtualKeyCode::F5 => Key::F5,
            winit::event::VirtualKeyCode::F6 => Key::F6,
            winit::event::VirtualKeyCode::F7 => Key::F7,
            winit::event::VirtualKeyCode::F8 => Key::F8,
            winit::event::VirtualKeyCode::F9 => Key::F9,
            winit::event::VirtualKeyCode::F10 => Key::F10,
            winit::event::VirtualKeyCode::F11 => Key::F11,
            winit::event::VirtualKeyCode::F12 => Key::F12,
            winit::event::VirtualKeyCode::F13 => Key::F13,
            winit::event::VirtualKeyCode::F14 => Key::F14,
            winit::event::VirtualKeyCode::F15 => Key::F15,
            winit::event::VirtualKeyCode::F16 => Key::F16,
            winit::event::VirtualKeyCode::F17 => Key::F17,
            winit::event::VirtualKeyCode::F18 => Key::F18,
            winit::event::VirtualKeyCode::F19 => Key::F19,
            winit::event::VirtualKeyCode::F20 => Key::F20,
            winit::event::VirtualKeyCode::F21 => Key::F21,
            winit::event::VirtualKeyCode::F22 => Key::F22,
            winit::event::VirtualKeyCode::F23 => Key::F23,
            winit::event::VirtualKeyCode::F24 => Key::F24,
            winit::event::VirtualKeyCode::Snapshot => Key::Snapshot,
            winit::event::VirtualKeyCode::Scroll => Key::Scroll,
            winit::event::VirtualKeyCode::Pause => Key::Pause,
            winit::event::VirtualKeyCode::Insert => Key::Insert,
            winit::event::VirtualKeyCode::Home => Key::Home,
            winit::event::VirtualKeyCode::Delete => Key::Delete,
            winit::event::VirtualKeyCode::End => Key::End,
            winit::event::VirtualKeyCode::PageDown => Key::PageDown,
            winit::event::VirtualKeyCode::PageUp => Key::PageUp,
            winit::event::VirtualKeyCode::Left => Key::Left,
            winit::event::VirtualKeyCode::Up => Key::Up,
            winit::event::VirtualKeyCode::Right => Key::Right,
            winit::event::VirtualKeyCode::Down => Key::Down,
            winit::event::VirtualKeyCode::Back => Key::Backspace,
            winit::event::VirtualKeyCode::Return => Key::Return,
            winit::event::VirtualKeyCode::Space => Key::Space,
            winit::event::VirtualKeyCode::Compose => Key::Compose,
            winit::event::VirtualKeyCode::Caret => Key::Caret,
            winit::event::VirtualKeyCode::Numlock => Key::Numlock,
            winit::event::VirtualKeyCode::Numpad0 => Key::Numpad0,
            winit::event::VirtualKeyCode::Numpad1 => Key::Numpad1,
            winit::event::VirtualKeyCode::Numpad2 => Key::Numpad2,
            winit::event::VirtualKeyCode::Numpad3 => Key::Numpad3,
            winit::event::VirtualKeyCode::Numpad4 => Key::Numpad4,
            winit::event::VirtualKeyCode::Numpad5 => Key::Numpad5,
            winit::event::VirtualKeyCode::Numpad6 => Key::Numpad6,
            winit::event::VirtualKeyCode::Numpad7 => Key::Numpad7,
            winit::event::VirtualKeyCode::Numpad8 => Key::Numpad8,
            winit::event::VirtualKeyCode::Numpad9 => Key::Numpad9,
            winit::event::VirtualKeyCode::NumpadAdd => Key::NumpadAdd,
            winit::event::VirtualKeyCode::NumpadDivide => Key::NumpadDivide,
            winit::event::VirtualKeyCode::NumpadDecimal => Key::NumpadDecimal,
            winit::event::VirtualKeyCode::NumpadComma => Key::NumpadComma,
            winit::event::VirtualKeyCode::NumpadEnter => Key::NumpadEnter,
            winit::event::VirtualKeyCode::NumpadEquals => Key::NumpadEquals,
            winit::event::VirtualKeyCode::NumpadMultiply => Key::NumpadMultiply,
            winit::event::VirtualKeyCode::NumpadSubtract => Key::NumpadSubtract,
            winit::event::VirtualKeyCode::AbntC1 => Key::AbntC1,
            winit::event::VirtualKeyCode::AbntC2 => Key::AbntC2,
            winit::event::VirtualKeyCode::Apostrophe => Key::Apostrophe,
            winit::event::VirtualKeyCode::Apps => Key::Apps,
            winit::event::VirtualKeyCode::Asterisk => Key::Asterisk,
            winit::event::VirtualKeyCode::At => Key::At,
            winit::event::VirtualKeyCode::Ax => Key::Ax,
            winit::event::VirtualKeyCode::Backslash => Key::Backslash,
            winit::event::VirtualKeyCode::Calculator => Key::Calculator,
            winit::event::VirtualKeyCode::Capital => Key::Capital,
            winit::event::VirtualKeyCode::Colon => Key::Colon,
            winit::event::VirtualKeyCode::Comma => Key::Comma,
            winit::event::VirtualKeyCode::Convert => Key::Convert,
            winit::event::VirtualKeyCode::Equals => Key::Equals,
            winit::event::VirtualKeyCode::Grave => Key::Grave,
            winit::event::VirtualKeyCode::Kana => Key::Kana,
            winit::event::VirtualKeyCode::Kanji => Key::Kanji,
            winit::event::VirtualKeyCode::LAlt => Key::LAlt,
            winit::event::VirtualKeyCode::LBracket => Key::LBracket,
            winit::event::VirtualKeyCode::LControl => Key::LControl,
            winit::event::VirtualKeyCode::LShift => Key::LShift,
            winit::event::VirtualKeyCode::LWin => Key::LWin,
            winit::event::VirtualKeyCode::Mail => Key::Mail,
            winit::event::VirtualKeyCode::MediaSelect => Key::MediaSelect,
            winit::event::VirtualKeyCode::MediaStop => Key::MediaStop,
            winit::event::VirtualKeyCode::Minus => Key::Minus,
            winit::event::VirtualKeyCode::Mute => Key::Mute,
            winit::event::VirtualKeyCode::MyComputer => Key::MyComputer,
            winit::event::VirtualKeyCode::NavigateForward => Key::NavigateForward,
            winit::event::VirtualKeyCode::NavigateBackward => Key::NavigateBackward,
            winit::event::VirtualKeyCode::NextTrack => Key::NextTrack,
            winit::event::VirtualKeyCode::NoConvert => Key::NoConvert,
            winit::event::VirtualKeyCode::OEM102 => Key::OEM102,
            winit::event::VirtualKeyCode::Period => Key::Period,
            winit::event::VirtualKeyCode::PlayPause => Key::PlayPause,
            winit::event::VirtualKeyCode::Plus => Key::Plus,
            winit::event::VirtualKeyCode::Power => Key::Power,
            winit::event::VirtualKeyCode::PrevTrack => Key::PrevTrack,
            winit::event::VirtualKeyCode::RAlt => Key::RAlt,
            winit::event::VirtualKeyCode::RBracket => Key::RBracket,
            winit::event::VirtualKeyCode::RControl => Key::RControl,
            winit::event::VirtualKeyCode::RShift => Key::RShift,
            winit::event::VirtualKeyCode::RWin => Key::RWin,
            winit::event::VirtualKeyCode::Semicolon => Key::Semicolon,
            winit::event::VirtualKeyCode::Slash => Key::Slash,
            winit::event::VirtualKeyCode::Sleep => Key::Sleep,
            winit::event::VirtualKeyCode::Stop => Key::Stop,
            winit::event::VirtualKeyCode::Sysrq => Key::Sysrq,
            winit::event::VirtualKeyCode::Tab => Key::Tab,
            winit::event::VirtualKeyCode::Underline => Key::Underline,
            winit::event::VirtualKeyCode::Unlabeled => Key::Unlabeled,
            winit::event::VirtualKeyCode::VolumeDown => Key::VolumeDown,
            winit::event::VirtualKeyCode::VolumeUp => Key::VolumeUp,
            winit::event::VirtualKeyCode::Wake => Key::Wake,
            winit::event::VirtualKeyCode::WebBack => Key::WebBack,
            winit::event::VirtualKeyCode::WebFavorites => Key::WebFavorites,
            winit::event::VirtualKeyCode::WebForward => Key::WebForward,
            winit::event::VirtualKeyCode::WebHome => Key::WebHome,
            winit::event::VirtualKeyCode::WebRefresh => Key::WebRefresh,
            winit::event::VirtualKeyCode::WebSearch => Key::WebSearch,
            winit::event::VirtualKeyCode::WebStop => Key::WebStop,
            winit::event::VirtualKeyCode::Yen => Key::Yen,
            winit::event::VirtualKeyCode::Copy => Key::Copy,
            winit::event::VirtualKeyCode::Paste => Key::Paste,
            winit::event::VirtualKeyCode::Cut => Key::Cut,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, EnumCount)]
pub enum Modifier {
    // Support for right modifiers will be added when winit does so
    LeftShift,
    LeftCtrl,
    LeftAlt,

    Meta,
}

#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct ModifierSet {
    modifiers: [bool; Modifier::COUNT],
}

impl ModifierSet {
    pub fn new(left_shift: bool, left_alt: bool, left_ctrl: bool, meta: bool) -> Self {
        let mut set = ModifierSet::default();
        set.modifiers[Modifier::LeftShift as usize] = left_shift;
        set.modifiers[Modifier::LeftAlt as usize] = left_alt;
        set.modifiers[Modifier::LeftCtrl as usize] = left_ctrl;
        set.modifiers[Modifier::Meta as usize] = meta;
        set
    }

    pub fn left_ctrl(&self) -> bool {
        self.modifiers[Modifier::LeftCtrl as usize]
    }
    pub fn left_alt(&self) -> bool {
        self.modifiers[Modifier::LeftAlt as usize]
    }
    pub fn left_shift(&self) -> bool {
        self.modifiers[Modifier::LeftShift as usize]
    }
    pub fn meta(&self) -> bool {
        self.modifiers[Modifier::Meta as usize]
    }
}

impl From<u32> for ModifierSet {
    fn from(bitmask: u32) -> Self {
        let mut set = ModifierSet::default();
        if bitmask & ModifiersState::SHIFT.bits() != 0 {
            set.modifiers[Modifier::LeftShift as usize] = true;
        }
        if bitmask & ModifiersState::ALT.bits() != 0 {
            set.modifiers[Modifier::LeftAlt as usize] = true;
        }
        if bitmask & ModifiersState::CTRL.bits() != 0 {
            set.modifiers[Modifier::LeftCtrl as usize] = true;
        }
        if bitmask & ModifiersState::LOGO.bits() != 0 {
            set.modifiers[Modifier::Meta as usize] = true;
        }

        set
    }
}
