// The following types are copied from winit, but without everything else winit comes with so that we can use this package in our guest code.

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use winit::keyboard::KeyCode;

/// Describes the appearance of the mouse cursor.
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, EnumString, Display, Default, Serialize, Deserialize,
)]
pub enum CursorIcon {
    /// The platform-dependent default cursor.
    #[default]
    Default,
    /// A simple crosshair.
    Crosshair,
    /// A hand (often used to indicate links in web browsers).
    Hand,
    /// Self explanatory.
    Arrow,
    /// Indicates something is to be moved.
    Move,
    /// Indicates text that may be selected or edited.
    Text,
    /// Program busy indicator.
    Wait,
    /// Help indicator (often rendered as a "?")
    Help,
    /// Progress indicator. Shows that processing is being done. But in contrast
    /// with "Wait" the user may still interact with the program. Often rendered
    /// as a spinning beach ball, or an arrow with a watch or hourglass.
    Progress,

    /// Cursor showing that something cannot be done.
    NotAllowed,
    ContextMenu,
    Cell,
    VerticalText,
    Alias,
    Copy,
    NoDrop,
    /// Indicates something can be grabbed.
    Grab,
    /// Indicates something is grabbed.
    Grabbing,
    AllScroll,
    ZoomIn,
    ZoomOut,

    /// Indicate that some edge is to be moved. For example, the 'SeResize' cursor
    /// is used when the movement starts from the south-east corner of the box.
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize,
}

#[cfg(feature = "native")]
impl From<CursorIcon> for winit::window::CursorIcon {
    fn from(value: CursorIcon) -> Self {
        match value {
            CursorIcon::Default => winit::window::CursorIcon::Default,
            CursorIcon::Crosshair => winit::window::CursorIcon::Crosshair,
            CursorIcon::Hand => winit::window::CursorIcon::Grab,
            CursorIcon::Arrow => winit::window::CursorIcon::Pointer,
            CursorIcon::Move => winit::window::CursorIcon::Move,
            CursorIcon::Text => winit::window::CursorIcon::Text,
            CursorIcon::Wait => winit::window::CursorIcon::Wait,
            CursorIcon::Help => winit::window::CursorIcon::Help,
            CursorIcon::Progress => winit::window::CursorIcon::Progress,
            CursorIcon::NotAllowed => winit::window::CursorIcon::NotAllowed,
            CursorIcon::ContextMenu => winit::window::CursorIcon::ContextMenu,
            CursorIcon::Cell => winit::window::CursorIcon::Cell,
            CursorIcon::VerticalText => winit::window::CursorIcon::VerticalText,
            CursorIcon::Alias => winit::window::CursorIcon::Alias,
            CursorIcon::Copy => winit::window::CursorIcon::Copy,
            CursorIcon::NoDrop => winit::window::CursorIcon::NoDrop,
            CursorIcon::Grab => winit::window::CursorIcon::Grab,
            CursorIcon::Grabbing => winit::window::CursorIcon::Grabbing,
            CursorIcon::AllScroll => winit::window::CursorIcon::AllScroll,
            CursorIcon::ZoomIn => winit::window::CursorIcon::ZoomIn,
            CursorIcon::ZoomOut => winit::window::CursorIcon::ZoomOut,
            CursorIcon::EResize => winit::window::CursorIcon::EResize,
            CursorIcon::NResize => winit::window::CursorIcon::NResize,
            CursorIcon::NeResize => winit::window::CursorIcon::NeResize,
            CursorIcon::NwResize => winit::window::CursorIcon::NwResize,
            CursorIcon::SResize => winit::window::CursorIcon::SResize,
            CursorIcon::SeResize => winit::window::CursorIcon::SeResize,
            CursorIcon::SwResize => winit::window::CursorIcon::SwResize,
            CursorIcon::WResize => winit::window::CursorIcon::WResize,
            CursorIcon::EwResize => winit::window::CursorIcon::EwResize,
            CursorIcon::NsResize => winit::window::CursorIcon::NsResize,
            CursorIcon::NeswResize => winit::window::CursorIcon::NeswResize,
            CursorIcon::NwseResize => winit::window::CursorIcon::NwseResize,
            CursorIcon::ColResize => winit::window::CursorIcon::ColResize,
            CursorIcon::RowResize => winit::window::CursorIcon::RowResize,
        }
    }
}

#[cfg(feature = "native")]
impl From<winit::window::CursorIcon> for CursorIcon {
    fn from(value: winit::window::CursorIcon) -> Self {
        match value {
            winit::window::CursorIcon::Default => CursorIcon::Default,
            winit::window::CursorIcon::Crosshair => CursorIcon::Crosshair,
            //winit::window::CursorIcon::Grab => CursorIcon::Hand,
            //winit::window::CursorIcon::Default => CursorIcon::Arrow,
            winit::window::CursorIcon::Move => CursorIcon::Move,
            winit::window::CursorIcon::Text => CursorIcon::Text,
            winit::window::CursorIcon::Wait => CursorIcon::Wait,
            winit::window::CursorIcon::Help => CursorIcon::Help,
            winit::window::CursorIcon::Progress => CursorIcon::Progress,
            winit::window::CursorIcon::NotAllowed => CursorIcon::NotAllowed,
            winit::window::CursorIcon::ContextMenu => CursorIcon::ContextMenu,
            winit::window::CursorIcon::Cell => CursorIcon::Cell,
            winit::window::CursorIcon::VerticalText => CursorIcon::VerticalText,
            winit::window::CursorIcon::Alias => CursorIcon::Alias,
            winit::window::CursorIcon::Copy => CursorIcon::Copy,
            winit::window::CursorIcon::NoDrop => CursorIcon::NoDrop,
            winit::window::CursorIcon::Grab => CursorIcon::Grab,
            winit::window::CursorIcon::Grabbing => CursorIcon::Grabbing,
            winit::window::CursorIcon::AllScroll => CursorIcon::AllScroll,
            winit::window::CursorIcon::ZoomIn => CursorIcon::ZoomIn,
            winit::window::CursorIcon::ZoomOut => CursorIcon::ZoomOut,
            winit::window::CursorIcon::EResize => CursorIcon::EResize,
            winit::window::CursorIcon::NResize => CursorIcon::NResize,
            winit::window::CursorIcon::NeResize => CursorIcon::NeResize,
            winit::window::CursorIcon::NwResize => CursorIcon::NwResize,
            winit::window::CursorIcon::SResize => CursorIcon::SResize,
            winit::window::CursorIcon::SeResize => CursorIcon::SeResize,
            winit::window::CursorIcon::SwResize => CursorIcon::SwResize,
            winit::window::CursorIcon::WResize => CursorIcon::WResize,
            winit::window::CursorIcon::EwResize => CursorIcon::EwResize,
            winit::window::CursorIcon::NsResize => CursorIcon::NsResize,
            winit::window::CursorIcon::NeswResize => CursorIcon::NeswResize,
            winit::window::CursorIcon::NwseResize => CursorIcon::NwseResize,
            winit::window::CursorIcon::ColResize => CursorIcon::ColResize,
            winit::window::CursorIcon::RowResize => CursorIcon::RowResize,
            winit::window::CursorIcon::Pointer => CursorIcon::Arrow,
            _ => todo!(),
        }
    }
}

/// Symbolic name for a keyboard key.
#[derive(
    Debug,
    Hash,
    Ord,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
    EnumString,
    Display,
    Serialize,
    Deserialize,
)]
#[repr(u32)]
pub enum VirtualKeyCode {
    /// The '1' key over the letters.
    Key1,
    /// The '2' key over the letters.
    Key2,
    /// The '3' key over the letters.
    Key3,
    /// The '4' key over the letters.
    Key4,
    /// The '5' key over the letters.
    Key5,
    /// The '6' key over the letters.
    Key6,
    /// The '7' key over the letters.
    Key7,
    /// The '8' key over the letters.
    Key8,
    /// The '9' key over the letters.
    Key9,
    /// The '0' key over the 'O' and 'P' keys.
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

    /// The Escape key, next to F1.
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

    /// Print Screen/SysRq.
    Snapshot,
    /// Scroll Lock.
    Scroll,
    /// Pause/Break key, next to Scroll lock.
    Pause,

    /// `Insert`, next to Backspace.
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

    /// The Backspace key, right over Enter.
    // TODO: rename
    Back,
    /// The Enter key.
    Return,
    /// The space bar.
    Space,

    /// The "Compose" key on Linux.
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
    // also called "Next"
    NavigateForward,
    // also called "Prior"
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

#[cfg(feature = "native")]
impl From<VirtualKeyCode> for KeyCode {
    fn from(value: VirtualKeyCode) -> Self {
        match value {
            VirtualKeyCode::Key1 => KeyCode::Digit1,
            VirtualKeyCode::Key2 => KeyCode::Digit2,
            VirtualKeyCode::Key3 => KeyCode::Digit3,
            VirtualKeyCode::Key4 => KeyCode::Digit4,
            VirtualKeyCode::Key5 => KeyCode::Digit5,
            VirtualKeyCode::Key6 => KeyCode::Digit6,
            VirtualKeyCode::Key7 => KeyCode::Digit7,
            VirtualKeyCode::Key8 => KeyCode::Digit8,
            VirtualKeyCode::Key9 => KeyCode::Digit9,
            VirtualKeyCode::Key0 => KeyCode::Digit0,
            VirtualKeyCode::A => KeyCode::KeyA,
            VirtualKeyCode::B => KeyCode::KeyB,
            VirtualKeyCode::C => KeyCode::KeyC,
            VirtualKeyCode::D => KeyCode::KeyD,
            VirtualKeyCode::E => KeyCode::KeyE,
            VirtualKeyCode::F => KeyCode::KeyF,
            VirtualKeyCode::G => KeyCode::KeyG,
            VirtualKeyCode::H => KeyCode::KeyH,
            VirtualKeyCode::I => KeyCode::KeyI,
            VirtualKeyCode::J => KeyCode::KeyJ,
            VirtualKeyCode::K => KeyCode::KeyK,
            VirtualKeyCode::L => KeyCode::KeyL,
            VirtualKeyCode::M => KeyCode::KeyM,
            VirtualKeyCode::N => KeyCode::KeyN,
            VirtualKeyCode::O => KeyCode::KeyO,
            VirtualKeyCode::P => KeyCode::KeyP,
            VirtualKeyCode::Q => KeyCode::KeyQ,
            VirtualKeyCode::R => KeyCode::KeyR,
            VirtualKeyCode::S => KeyCode::KeyS,
            VirtualKeyCode::T => KeyCode::KeyT,
            VirtualKeyCode::U => KeyCode::KeyU,
            VirtualKeyCode::V => KeyCode::KeyV,
            VirtualKeyCode::W => KeyCode::KeyW,
            VirtualKeyCode::X => KeyCode::KeyX,
            VirtualKeyCode::Y => KeyCode::KeyY,
            VirtualKeyCode::Z => KeyCode::KeyZ,
            VirtualKeyCode::Escape => KeyCode::Escape,
            VirtualKeyCode::F1 => KeyCode::F1,
            VirtualKeyCode::F2 => KeyCode::F2,
            VirtualKeyCode::F3 => KeyCode::F3,
            VirtualKeyCode::F4 => KeyCode::F4,
            VirtualKeyCode::F5 => KeyCode::F5,
            VirtualKeyCode::F6 => KeyCode::F6,
            VirtualKeyCode::F7 => KeyCode::F7,
            VirtualKeyCode::F8 => KeyCode::F8,
            VirtualKeyCode::F9 => KeyCode::F9,
            VirtualKeyCode::F10 => KeyCode::F10,
            VirtualKeyCode::F11 => KeyCode::F11,
            VirtualKeyCode::F12 => KeyCode::F12,
            VirtualKeyCode::F13 => KeyCode::F13,
            VirtualKeyCode::F14 => KeyCode::F14,
            VirtualKeyCode::F15 => KeyCode::F15,
            VirtualKeyCode::F16 => KeyCode::F16,
            VirtualKeyCode::F17 => KeyCode::F17,
            VirtualKeyCode::F18 => KeyCode::F18,
            VirtualKeyCode::F19 => KeyCode::F19,
            VirtualKeyCode::F20 => KeyCode::F20,
            VirtualKeyCode::F21 => KeyCode::F21,
            VirtualKeyCode::F22 => KeyCode::F22,
            VirtualKeyCode::F23 => KeyCode::F23,
            VirtualKeyCode::F24 => KeyCode::F24,
            // VirtualKeyCode::Snapshot => KeyCode::Snapshot,
            // VirtualKeyCode::Scroll => KeyCode::Scroll,
            VirtualKeyCode::Pause => KeyCode::Pause,
            VirtualKeyCode::Insert => KeyCode::Insert,
            VirtualKeyCode::Home => KeyCode::Home,
            VirtualKeyCode::Delete => KeyCode::Delete,
            VirtualKeyCode::End => KeyCode::End,
            VirtualKeyCode::PageDown => KeyCode::PageDown,
            VirtualKeyCode::PageUp => KeyCode::PageUp,
            VirtualKeyCode::Left => KeyCode::ArrowLeft,
            VirtualKeyCode::Up => KeyCode::ArrowUp,
            VirtualKeyCode::Right => KeyCode::ArrowRight,
            VirtualKeyCode::Down => KeyCode::ArrowDown,
            VirtualKeyCode::Back => KeyCode::Backspace,
            VirtualKeyCode::Return => KeyCode::Enter,
            VirtualKeyCode::Space => KeyCode::Space,
            //VirtualKeyCode::Compose => KeyCode::Compose,
            //VirtualKeyCode::Caret => KeyCode::Caret,
            VirtualKeyCode::Numlock => KeyCode::NumLock,
            VirtualKeyCode::Numpad0 => KeyCode::Numpad0,
            VirtualKeyCode::Numpad1 => KeyCode::Numpad1,
            VirtualKeyCode::Numpad2 => KeyCode::Numpad2,
            VirtualKeyCode::Numpad3 => KeyCode::Numpad3,
            VirtualKeyCode::Numpad4 => KeyCode::Numpad4,
            VirtualKeyCode::Numpad5 => KeyCode::Numpad5,
            VirtualKeyCode::Numpad6 => KeyCode::Numpad6,
            VirtualKeyCode::Numpad7 => KeyCode::Numpad7,
            VirtualKeyCode::Numpad8 => KeyCode::Numpad8,
            VirtualKeyCode::Numpad9 => KeyCode::Numpad9,
            VirtualKeyCode::NumpadAdd => KeyCode::NumpadAdd,
            VirtualKeyCode::NumpadDivide => KeyCode::NumpadDivide,
            VirtualKeyCode::NumpadDecimal => KeyCode::NumpadDecimal,
            VirtualKeyCode::NumpadComma => KeyCode::NumpadComma,
            VirtualKeyCode::NumpadEnter => KeyCode::NumpadEnter,
            VirtualKeyCode::NumpadEquals => KeyCode::NumpadEqual,
            VirtualKeyCode::NumpadMultiply => KeyCode::NumpadMultiply,
            VirtualKeyCode::NumpadSubtract => KeyCode::NumpadSubtract,
            VirtualKeyCode::AbntC1 => KeyCode::AltLeft,
            VirtualKeyCode::AbntC2 => KeyCode::AltRight,
            VirtualKeyCode::Apostrophe => KeyCode::Quote,
            //VirtualKeyCode::Apps => KeyCode::Apps,
            VirtualKeyCode::Asterisk => KeyCode::NumpadStar,
            VirtualKeyCode::At => KeyCode::BracketLeft,
            VirtualKeyCode::Ax => KeyCode::BracketRight,
            VirtualKeyCode::Backslash => KeyCode::Backslash,
            VirtualKeyCode::Calculator => KeyCode::LaunchApp1,
            VirtualKeyCode::Capital => KeyCode::CapsLock,
            VirtualKeyCode::Colon => KeyCode::Semicolon,
            VirtualKeyCode::Comma => KeyCode::Comma,
            VirtualKeyCode::Convert => KeyCode::Convert,
            VirtualKeyCode::Equals => KeyCode::Equal,
            VirtualKeyCode::Grave => KeyCode::Backquote,
            VirtualKeyCode::Kana => KeyCode::KanaMode,
            //VirtualKeyCode::Kanji => KeyCode::KanjiMode,
            VirtualKeyCode::LAlt => KeyCode::AltLeft,
            VirtualKeyCode::LBracket => KeyCode::BracketLeft,
            VirtualKeyCode::LControl => KeyCode::ControlLeft,
            VirtualKeyCode::LShift => KeyCode::ShiftLeft,
            VirtualKeyCode::LWin => KeyCode::SuperLeft,
            VirtualKeyCode::Mail => KeyCode::LaunchMail,
            VirtualKeyCode::MediaSelect => KeyCode::MediaSelect,
            VirtualKeyCode::MediaStop => KeyCode::MediaStop,
            VirtualKeyCode::Minus => KeyCode::Minus,
            VirtualKeyCode::Mute => KeyCode::AudioVolumeMute,
            VirtualKeyCode::MyComputer => KeyCode::LaunchApp1,
            // VirtualKeyCode::NavigateForward => KeyCode::NavigateForward,
            // VirtualKeyCode::NavigateBackward => KeyCode::NavigateBackward,
            // VirtualKeyCode::NextTrack => KeyCode::NextTrack,
            // VirtualKeyCode::NoConvert => KeyCode::NoConvert,
            // VirtualKeyCode::OEM102 => KeyCode::OEM102,
            // VirtualKeyCode::Period => KeyCode::Period,
            // VirtualKeyCode::PlayPause => KeyCode::PlayPause,
            // VirtualKeyCode::Plus => KeyCode::Plus,
            // VirtualKeyCode::Power => KeyCode::Power,
            // VirtualKeyCode::PrevTrack => KeyCode::PrevTrack,
            // VirtualKeyCode::RAlt => KeyCode::RAlt,
            // VirtualKeyCode::RBracket => KeyCode::RBracket,
            // VirtualKeyCode::RControl => KeyCode::RControl,
            // VirtualKeyCode::RShift => KeyCode::RShift,
            // VirtualKeyCode::RWin => KeyCode::RWin,
            // VirtualKeyCode::Semicolon => KeyCode::Semicolon,
            // VirtualKeyCode::Slash => KeyCode::Slash,
            // VirtualKeyCode::Sleep => KeyCode::Sleep,
            // VirtualKeyCode::Stop => KeyCode::Stop,
            // VirtualKeyCode::Sysrq => KeyCode::Sysrq,
            // VirtualKeyCode::Tab => KeyCode::Tab,
            // VirtualKeyCode::Underline => KeyCode::Underline,
            // VirtualKeyCode::Unlabeled => KeyCode::Unlabeled,
            // VirtualKeyCode::VolumeDown => KeyCode::VolumeDown,
            // VirtualKeyCode::VolumeUp => KeyCode::VolumeUp,
            // VirtualKeyCode::Wake => KeyCode::Wake,
            // VirtualKeyCode::WebBack => KeyCode::WebBack,
            // VirtualKeyCode::WebFavorites => KeyCode::WebFavorites,
            // VirtualKeyCode::WebForward => KeyCode::WebForward,
            // VirtualKeyCode::WebHome => KeyCode::WebHome,
            // VirtualKeyCode::WebRefresh => KeyCode::WebRefresh,
            // VirtualKeyCode::WebSearch => KeyCode::WebSearch,
            // VirtualKeyCode::WebStop => KeyCode::WebStop,
            // VirtualKeyCode::Yen => KeyCode::Yen,
            VirtualKeyCode::Copy => KeyCode::Copy,
            VirtualKeyCode::Paste => KeyCode::Paste,
            VirtualKeyCode::Cut => KeyCode::Cut,
            _=>KeyCode::AltLeft
        }
    }
}

#[cfg(feature = "native")]
impl From<KeyCode> for VirtualKeyCode {
    fn from(value: KeyCode) -> Self {
        match value {
            KeyCode::Digit1 => VirtualKeyCode::Key1,
            KeyCode::Digit2 => VirtualKeyCode::Key2,
            KeyCode::Digit3 => VirtualKeyCode::Key3,
            KeyCode::Digit4 => VirtualKeyCode::Key4,
            KeyCode::Digit5 => VirtualKeyCode::Key5,
            KeyCode::Digit6 => VirtualKeyCode::Key6,
            KeyCode::Digit7 => VirtualKeyCode::Key7,
            KeyCode::Digit8 => VirtualKeyCode::Key8,
            KeyCode::Digit9 => VirtualKeyCode::Key9,
            KeyCode::Digit0 => VirtualKeyCode::Key0,
            KeyCode::KeyA => VirtualKeyCode::A,
            KeyCode::KeyB => VirtualKeyCode::B,
            KeyCode::KeyC => VirtualKeyCode::C,
            KeyCode::KeyD => VirtualKeyCode::D,
            KeyCode::KeyE => VirtualKeyCode::E,
            KeyCode::KeyF => VirtualKeyCode::F,
            KeyCode::KeyG => VirtualKeyCode::G,
            KeyCode::KeyH => VirtualKeyCode::H,
            KeyCode::KeyI => VirtualKeyCode::I,
            KeyCode::KeyJ => VirtualKeyCode::J,
            KeyCode::KeyK => VirtualKeyCode::K,
            KeyCode::KeyL => VirtualKeyCode::L,
            KeyCode::KeyM => VirtualKeyCode::M,
            KeyCode::KeyN => VirtualKeyCode::N,
            KeyCode::KeyO => VirtualKeyCode::O,
            KeyCode::KeyP => VirtualKeyCode::P,
            KeyCode::KeyQ => VirtualKeyCode::Q,
            KeyCode::KeyR => VirtualKeyCode::R,
            KeyCode::KeyS => VirtualKeyCode::S,
            KeyCode::KeyT => VirtualKeyCode::T,
            KeyCode::KeyU => VirtualKeyCode::U,
            KeyCode::KeyV => VirtualKeyCode::V,
            KeyCode::KeyW => VirtualKeyCode::W,
            KeyCode::KeyX => VirtualKeyCode::X,
            KeyCode::KeyY => VirtualKeyCode::Y,
            KeyCode::KeyZ => VirtualKeyCode::Z,
            KeyCode::Escape => VirtualKeyCode::Escape,
            KeyCode::F1 => VirtualKeyCode::F1,
            KeyCode::F2 => VirtualKeyCode::F2,
            KeyCode::F3 => VirtualKeyCode::F3,
            KeyCode::F4 => VirtualKeyCode::F4,
            KeyCode::F5 => VirtualKeyCode::F5,
            KeyCode::F6 => VirtualKeyCode::F6,
            KeyCode::F7 => VirtualKeyCode::F7,
            KeyCode::F8 => VirtualKeyCode::F8,
            KeyCode::F9 => VirtualKeyCode::F9,
            KeyCode::F10 => VirtualKeyCode::F10,
            KeyCode::F11 => VirtualKeyCode::F11,
            KeyCode::F12 => VirtualKeyCode::F12,
            KeyCode::F13 => VirtualKeyCode::F13,
            KeyCode::F14 => VirtualKeyCode::F14,
            KeyCode::F15 => VirtualKeyCode::F15,
            KeyCode::F16 => VirtualKeyCode::F16,
            KeyCode::F17 => VirtualKeyCode::F17,
            KeyCode::F18 => VirtualKeyCode::F18,
            KeyCode::F19 => VirtualKeyCode::F19,
            KeyCode::F20 => VirtualKeyCode::F20,
            KeyCode::F21 => VirtualKeyCode::F21,
            KeyCode::F22 => VirtualKeyCode::F22,
            KeyCode::F23 => VirtualKeyCode::F23,
            KeyCode::F24 => VirtualKeyCode::F24,
            //KeyCode::Snapshot => VirtualKeyCode::Snapshot,
            KeyCode::ScrollLock => VirtualKeyCode::Scroll,
            KeyCode::Pause => VirtualKeyCode::Pause,
            KeyCode::Insert => VirtualKeyCode::Insert,
            KeyCode::Home => VirtualKeyCode::Home,
            KeyCode::Delete => VirtualKeyCode::Delete,
            KeyCode::End => VirtualKeyCode::End,
            KeyCode::PageDown => VirtualKeyCode::PageDown,
            KeyCode::PageUp => VirtualKeyCode::PageUp,
            KeyCode::ArrowLeft => VirtualKeyCode::Left,
            KeyCode::ArrowUp => VirtualKeyCode::Up,
            KeyCode::ArrowRight => VirtualKeyCode::Right,
            KeyCode::ArrowDown => VirtualKeyCode::Down,
            KeyCode::Backspace => VirtualKeyCode::Back,
            KeyCode::Enter => VirtualKeyCode::Return,
            KeyCode::Space => VirtualKeyCode::Space,
            // KeyCode::Compose => VirtualKeyCode::Compose,
            // KeyCode::Caret => VirtualKeyCode::Caret,
            KeyCode::NumLock => VirtualKeyCode::Numlock,
            KeyCode::Numpad0 => VirtualKeyCode::Numpad0,
            KeyCode::Numpad1 => VirtualKeyCode::Numpad1,
            KeyCode::Numpad2 => VirtualKeyCode::Numpad2,
            KeyCode::Numpad3 => VirtualKeyCode::Numpad3,
            KeyCode::Numpad4 => VirtualKeyCode::Numpad4,
            KeyCode::Numpad5 => VirtualKeyCode::Numpad5,
            KeyCode::Numpad6 => VirtualKeyCode::Numpad6,
            KeyCode::Numpad7 => VirtualKeyCode::Numpad7,
            KeyCode::Numpad8 => VirtualKeyCode::Numpad8,
            KeyCode::Numpad9 => VirtualKeyCode::Numpad9,
            KeyCode::NumpadAdd => VirtualKeyCode::NumpadAdd,
            KeyCode::NumpadDivide => VirtualKeyCode::NumpadDivide,
            KeyCode::NumpadDecimal => VirtualKeyCode::NumpadDecimal,
            KeyCode::NumpadComma => VirtualKeyCode::NumpadComma,
            //123
            // KeyCode::NumpadEnter => VirtualKeyCode::NumpadEnter,
            // KeyCode::NumpadEquals => VirtualKeyCode::NumpadEquals,
            // KeyCode::NumpadMultiply => VirtualKeyCode::NumpadMultiply,
            // KeyCode::NumpadSubtract => VirtualKeyCode::NumpadSubtract,
            // KeyCode::AbntC1 => VirtualKeyCode::AbntC1,
            // KeyCode::AbntC2 => VirtualKeyCode::AbntC2,
            // KeyCode::Apostrophe => VirtualKeyCode::Apostrophe,
            // KeyCode::Apps => VirtualKeyCode::Apps,
            // KeyCode::Asterisk => VirtualKeyCode::Asterisk,
            // KeyCode::At => VirtualKeyCode::At,
            // KeyCode::Ax => VirtualKeyCode::Ax,
            // KeyCode::Backslash => VirtualKeyCode::Backslash,
            // KeyCode::Calculator => VirtualKeyCode::Calculator,
            // KeyCode::Capital => VirtualKeyCode::Capital,
            // KeyCode::Colon => VirtualKeyCode::Colon,
            // KeyCode::Comma => VirtualKeyCode::Comma,
            // KeyCode::Convert => VirtualKeyCode::Convert,
            // KeyCode::Equals => VirtualKeyCode::Equals,
            // KeyCode::Grave => VirtualKeyCode::Grave,
            // KeyCode::Kana => VirtualKeyCode::Kana,
            // KeyCode::Kanji => VirtualKeyCode::Kanji,
            // KeyCode::LAlt => VirtualKeyCode::LAlt,
            // KeyCode::LBracket => VirtualKeyCode::LBracket,
            // KeyCode::LControl => VirtualKeyCode::LControl,
            // KeyCode::LShift => VirtualKeyCode::LShift,
            // KeyCode::LWin => VirtualKeyCode::LWin,
            // KeyCode::Mail => VirtualKeyCode::Mail,
            // KeyCode::MediaSelect => VirtualKeyCode::MediaSelect,
            // KeyCode::MediaStop => VirtualKeyCode::MediaStop,
            // KeyCode::Minus => VirtualKeyCode::Minus,
            // KeyCode::Mute => VirtualKeyCode::Mute,
            // KeyCode::MyComputer => VirtualKeyCode::MyComputer,
            // KeyCode::NavigateForward => VirtualKeyCode::NavigateForward,
            // KeyCode::NavigateBackward => VirtualKeyCode::NavigateBackward,
            // KeyCode::NextTrack => VirtualKeyCode::NextTrack,
            // KeyCode::NoConvert => VirtualKeyCode::NoConvert,
            // KeyCode::OEM102 => VirtualKeyCode::OEM102,
            // KeyCode::Period => VirtualKeyCode::Period,
            // KeyCode::PlayPause => VirtualKeyCode::PlayPause,
            // KeyCode::Plus => VirtualKeyCode::Plus,
            // KeyCode::Power => VirtualKeyCode::Power,
            // KeyCode::PrevTrack => VirtualKeyCode::PrevTrack,
            // KeyCode::RAlt => VirtualKeyCode::RAlt,
            // KeyCode::RBracket => VirtualKeyCode::RBracket,
            // KeyCode::RControl => VirtualKeyCode::RControl,
            // KeyCode::RShift => VirtualKeyCode::RShift,
            // KeyCode::RWin => VirtualKeyCode::RWin,
            // KeyCode::Semicolon => VirtualKeyCode::Semicolon,
            // KeyCode::Slash => VirtualKeyCode::Slash,
            // KeyCode::Sleep => VirtualKeyCode::Sleep,
            // KeyCode::Stop => VirtualKeyCode::Stop,
            // KeyCode::Sysrq => VirtualKeyCode::Sysrq,
            // KeyCode::Tab => VirtualKeyCode::Tab,
            // KeyCode::Underline => VirtualKeyCode::Underline,
            // KeyCode::Unlabeled => VirtualKeyCode::Unlabeled,
            // KeyCode::VolumeDown => VirtualKeyCode::VolumeDown,
            // KeyCode::VolumeUp => VirtualKeyCode::VolumeUp,
            // KeyCode::Wake => VirtualKeyCode::Wake,
            // KeyCode::WebBack => VirtualKeyCode::WebBack,
            // KeyCode::WebFavorites => VirtualKeyCode::WebFavorites,
            // KeyCode::WebForward => VirtualKeyCode::WebForward,
            // KeyCode::WebHome => VirtualKeyCode::WebHome,
            // KeyCode::WebRefresh => VirtualKeyCode::WebRefresh,
            // KeyCode::WebSearch => VirtualKeyCode::WebSearch,
            // KeyCode::WebStop => VirtualKeyCode::WebStop,
            // KeyCode::Yen => VirtualKeyCode::Yen,
            // KeyCode::Copy => VirtualKeyCode::Copy,
            // KeyCode::Paste => VirtualKeyCode::Paste,
            // KeyCode::Cut => VirtualKeyCode::Cut,
            _=>VirtualKeyCode::A
        }
    }
}

impl ModifiersState {
    /// Returns `true` if the shift key is pressed.
    pub fn shift(&self) -> bool {
        self.intersects(Self::SHIFT)
    }
    /// Returns `true` if the control key is pressed.
    pub fn ctrl(&self) -> bool {
        self.intersects(Self::CTRL)
    }
    /// Returns `true` if the alt key is pressed.
    pub fn alt(&self) -> bool {
        self.intersects(Self::ALT)
    }
    /// Returns `true` if the logo key is pressed.
    pub fn logo(&self) -> bool {
        self.intersects(Self::LOGO)
    }
}

bitflags! {
    /// Represents the current state of the keyboard modifiers
    ///
    /// Each flag represents a modifier and is set if this modifier is active.
    #[derive(Default)]
    pub struct ModifiersState: u32 {
        // left and right modifiers are currently commented out, but we should be able to support
        // them in a future release
        /// The "shift" key.
        const SHIFT = 0b100;
        // const LSHIFT = 0b010;
        // const RSHIFT = 0b001;
        /// The "control" key.
        const CTRL = 0b100 << 3;
        // const LCTRL = 0b010 << 3;
        // const RCTRL = 0b001 << 3;
        /// The "alt" key.
        const ALT = 0b100 << 6;
        // const LALT = 0b010 << 6;
        // const RALT = 0b001 << 6;
        /// This is the "windows" key on PC and "command" key on Mac.
        const LOGO = 0b100 << 9;
        // const LLOGO = 0b010 << 9;
        // const RLOGO = 0b001 << 9;
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}
impl From<u32> for MouseButton {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Left,
            1 => Self::Right,
            2 => Self::Middle,
            x => Self::Other(x as _),
        }
    }
}
impl From<MouseButton> for u32 {
    fn from(value: MouseButton) -> Self {
        match value {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::Other(x) => x as _,
        }
    }
}
#[cfg(feature = "native")]
impl From<winit::event::MouseButton> for MouseButton {
    fn from(value: winit::event::MouseButton) -> Self {
        match value {
            winit::event::MouseButton::Left => Self::Left,
            winit::event::MouseButton::Right => Self::Right,
            winit::event::MouseButton::Middle => Self::Middle,
            winit::event::MouseButton::Other(x) => Self::Other(x),
            //123
            _=>Self::Left
        }
    }
}
#[cfg(feature = "native")]
impl From<MouseButton> for winit::event::MouseButton {
    fn from(value: MouseButton) -> Self {
        match value {
            MouseButton::Left => Self::Left,
            MouseButton::Right => Self::Right,
            MouseButton::Middle => Self::Middle,
            MouseButton::Other(x) => Self::Other(x),
        }
    }
}
