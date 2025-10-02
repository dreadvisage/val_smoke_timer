use anyhow::{Context, Result};
use directories::BaseDirs;
use rdev::{Button, Key};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;

const PROGRAM_DIR_NAME: &str = env!("CARGO_PKG_NAME");
const PROGRAM_CONFIG_NAME: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub initial_pos: (f32, f32),
    pub start_key: String,
    pub cancelable_keys: Vec<String>,
    pub confirm_key: String,
    pub timer_start: f32,
    pub max_timers: usize,
    pub subtext_string: String,
    pub show_subtext: bool,
    pub show_numbering: bool,
    pub add_new_on_left: bool,
    pub overwrite_oldest: bool,
    pub enable_red_text: bool,
    pub red_text_threshold: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            initial_pos: (0.0, 0.0),
            start_key: "Key:KeyE".to_string(),
            cancelable_keys: vec![
                "Key:KeyE".to_string(),
                "Key:KeyQ".to_string(),
                "Key:KeyC".to_string(),
                "Key:KeyX".to_string(),
                "Key:Escape".to_string(),
                "Key:Num1".to_string(),
                "Key:Num2".to_string(),
                "Key:Num3".to_string(),
                "Key:Num4".to_string(),
            ],
            confirm_key: "Mouse:Right".to_string(),
            timer_start: 19.25,
            max_timers: 3,
            subtext_string: "".to_string(),
            show_subtext: true,
            show_numbering: true,
            add_new_on_left: true,
            overwrite_oldest: false,
            enable_red_text: true,
            red_text_threshold: 5.0,
        }
    }
}

impl Config {
    /// Get the default config file path
    pub fn get_default_config_path() -> Result<PathBuf> {
        let base_dirs = BaseDirs::new().with_context(|| "Failed to get base dirs")?;

        Ok(base_dirs
            .config_local_dir()
            .join(PROGRAM_DIR_NAME)
            .join(PROGRAM_CONFIG_NAME))
    }

    /// Load config from file, or return default if it doesn't exist or fails to parse
    pub fn load() -> Self {
        match Self::try_load() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config: {e:?}. Using and saving defaults");
                let default_config = Self::default();
                // Save the default config
                if let Err(save_err) = default_config.save() {
                    eprintln!("Failed to save default config: {save_err:?}");
                }
                default_config
            }
        }
    }

    fn try_load() -> Result<Self> {
        let path = Self::get_default_config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {path:?}"))?;

        let config: Config =
            toml::from_str(&contents).with_context(|| "Failed to parse config file")?;

        Ok(config)
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let path = Self::get_default_config_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {parent:?}"))?;
        }

        let contents =
            toml::to_string_pretty(self).with_context(|| "Failed to serialize config")?;

        fs::write(&path, contents)
            .with_context(|| format!("Failed to write config file: {path:?}"))?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputBinding {
    Key(Key),
    Mouse(Button),
}

impl InputBinding {
    pub fn from_string(s: &str) -> Option<Self> {
        if let Some(key_str) = s.strip_prefix("Key:") {
            string_to_key(key_str).map(InputBinding::Key)
        } else if let Some(button_str) = s.strip_prefix("Mouse:") {
            string_to_button(button_str).map(InputBinding::Mouse)
        } else {
            // Legacy support for old configs
            if s == "RightMouse" {
                Some(InputBinding::Mouse(Button::Right))
            } else {
                string_to_key(s).map(InputBinding::Key)
            }
        }
    }
}

impl Display for InputBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputBinding::Key(key) => write!(f, "Key:{}", key_to_string(key)),
            InputBinding::Mouse(button) => write!(f, "Mouse:{}", button_to_string(button)),
        }
    }
}

/// Convert Key enum to string representation
pub fn key_to_string(key: &Key) -> String {
    match key {
        Key::Alt => "Alt",
        Key::AltGr => "AltGr",
        Key::Backspace => "Backspace",
        Key::CapsLock => "CapsLock",
        Key::ControlLeft => "ControlLeft",
        Key::ControlRight => "ControlRight",
        Key::Delete => "Delete",
        Key::DownArrow => "DownArrow",
        Key::End => "End",
        Key::Escape => "Escape",
        Key::F1 => "F1",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",
        Key::F13 => "F13",
        Key::F14 => "F14",
        Key::F15 => "F15",
        Key::F16 => "F16",
        Key::F17 => "F17",
        Key::F18 => "F18",
        Key::F19 => "F19",
        Key::F20 => "F20",
        Key::F21 => "F21",
        Key::F22 => "F22",
        Key::F23 => "F23",
        Key::F24 => "F24",
        Key::Home => "Home",
        Key::LeftArrow => "LeftArrow",
        Key::MetaLeft => "MetaLeft",
        Key::MetaRight => "MetaRight",
        Key::PageDown => "PageDown",
        Key::PageUp => "PageUp",
        Key::Return => "Return",
        Key::RightArrow => "RightArrow",
        Key::ShiftLeft => "ShiftLeft",
        Key::ShiftRight => "ShiftRight",
        Key::Space => "Space",
        Key::Tab => "Tab",
        Key::UpArrow => "UpArrow",
        Key::PrintScreen => "PrintScreen",
        Key::ScrollLock => "ScrollLock",
        Key::Pause => "Pause",
        Key::NumLock => "NumLock",
        Key::BackQuote => "BackQuote",
        Key::Num1 => "Num1",
        Key::Num2 => "Num2",
        Key::Num3 => "Num3",
        Key::Num4 => "Num4",
        Key::Num5 => "Num5",
        Key::Num6 => "Num6",
        Key::Num7 => "Num7",
        Key::Num8 => "Num8",
        Key::Num9 => "Num9",
        Key::Num0 => "Num0",
        Key::Minus => "Minus",
        Key::Equal => "Equal",
        Key::KeyQ => "KeyQ",
        Key::KeyW => "KeyW",
        Key::KeyE => "KeyE",
        Key::KeyR => "KeyR",
        Key::KeyT => "KeyT",
        Key::KeyY => "KeyY",
        Key::KeyU => "KeyU",
        Key::KeyI => "KeyI",
        Key::KeyO => "KeyO",
        Key::KeyP => "KeyP",
        Key::LeftBracket => "LeftBracket",
        Key::RightBracket => "RightBracket",
        Key::KeyA => "KeyA",
        Key::KeyS => "KeyS",
        Key::KeyD => "KeyD",
        Key::KeyF => "KeyF",
        Key::KeyG => "KeyG",
        Key::KeyH => "KeyH",
        Key::KeyJ => "KeyJ",
        Key::KeyK => "KeyK",
        Key::KeyL => "KeyL",
        Key::SemiColon => "SemiColon",
        Key::Quote => "Quote",
        Key::BackSlash => "BackSlash",
        Key::IntlBackslash => "IntlBackslash",
        Key::KeyZ => "KeyZ",
        Key::KeyX => "KeyX",
        Key::KeyC => "KeyC",
        Key::KeyV => "KeyV",
        Key::KeyB => "KeyB",
        Key::KeyN => "KeyN",
        Key::KeyM => "KeyM",
        Key::Comma => "Comma",
        Key::Dot => "Dot",
        Key::Slash => "Slash",
        Key::Insert => "Insert",
        Key::KpReturn => "KpReturn",
        Key::KpMinus => "KpMinus",
        Key::KpPlus => "KpPlus",
        Key::KpMultiply => "KpMultiply",
        Key::KpDivide => "KpDivide",
        Key::Kp0 => "Kp0",
        Key::Kp1 => "Kp1",
        Key::Kp2 => "Kp2",
        Key::Kp3 => "Kp3",
        Key::Kp4 => "Kp4",
        Key::Kp5 => "Kp5",
        Key::Kp6 => "Kp6",
        Key::Kp7 => "Kp7",
        Key::Kp8 => "Kp8",
        Key::Kp9 => "Kp9",
        Key::KpDelete => "KpDelete",
        Key::Function => "Function",
        Key::VolumeUp => "VolumeUp",
        Key::VolumeDown => "VolumeDown",
        Key::VolumeMute => "VolumeMute",
        Key::BrightnessUp => "BrightnessUp",
        Key::BrightnessDown => "BrightnessDown",
        Key::PreviousTrack => "PreviousTrack",
        Key::PlayPause => "PlayPause",
        Key::PlayCd => "PlayCd",
        Key::NextTrack => "NextTrack",
        Key::Unknown(code) => return format!("Unknown({code})"),
    }
    .to_string()
}

/// Convert string to Key enum
pub fn string_to_key(s: &str) -> Option<Key> {
    match s {
        "Alt" => Some(Key::Alt),
        "AltGr" => Some(Key::AltGr),
        "Backspace" => Some(Key::Backspace),
        "CapsLock" => Some(Key::CapsLock),
        "ControlLeft" => Some(Key::ControlLeft),
        "ControlRight" => Some(Key::ControlRight),
        "Delete" => Some(Key::Delete),
        "DownArrow" => Some(Key::DownArrow),
        "End" => Some(Key::End),
        "Escape" => Some(Key::Escape),
        "F1" => Some(Key::F1),
        "F2" => Some(Key::F2),
        "F3" => Some(Key::F3),
        "F4" => Some(Key::F4),
        "F5" => Some(Key::F5),
        "F6" => Some(Key::F6),
        "F7" => Some(Key::F7),
        "F8" => Some(Key::F8),
        "F9" => Some(Key::F9),
        "F10" => Some(Key::F10),
        "F11" => Some(Key::F11),
        "F12" => Some(Key::F12),
        "F13" => Some(Key::F13),
        "F14" => Some(Key::F14),
        "F15" => Some(Key::F15),
        "F16" => Some(Key::F16),
        "F17" => Some(Key::F17),
        "F18" => Some(Key::F18),
        "F19" => Some(Key::F19),
        "F20" => Some(Key::F20),
        "F21" => Some(Key::F21),
        "F22" => Some(Key::F22),
        "F23" => Some(Key::F23),
        "F24" => Some(Key::F24),
        "Home" => Some(Key::Home),
        "LeftArrow" => Some(Key::LeftArrow),
        "MetaLeft" => Some(Key::MetaLeft),
        "MetaRight" => Some(Key::MetaRight),
        "PageDown" => Some(Key::PageDown),
        "PageUp" => Some(Key::PageUp),
        "Return" => Some(Key::Return),
        "RightArrow" => Some(Key::RightArrow),
        "ShiftLeft" => Some(Key::ShiftLeft),
        "ShiftRight" => Some(Key::ShiftRight),
        "Space" => Some(Key::Space),
        "Tab" => Some(Key::Tab),
        "UpArrow" => Some(Key::UpArrow),
        "PrintScreen" => Some(Key::PrintScreen),
        "ScrollLock" => Some(Key::ScrollLock),
        "Pause" => Some(Key::Pause),
        "NumLock" => Some(Key::NumLock),
        "BackQuote" => Some(Key::BackQuote),
        "Num1" => Some(Key::Num1),
        "Num2" => Some(Key::Num2),
        "Num3" => Some(Key::Num3),
        "Num4" => Some(Key::Num4),
        "Num5" => Some(Key::Num5),
        "Num6" => Some(Key::Num6),
        "Num7" => Some(Key::Num7),
        "Num8" => Some(Key::Num8),
        "Num9" => Some(Key::Num9),
        "Num0" => Some(Key::Num0),
        "Minus" => Some(Key::Minus),
        "Equal" => Some(Key::Equal),
        "KeyQ" => Some(Key::KeyQ),
        "KeyW" => Some(Key::KeyW),
        "KeyE" => Some(Key::KeyE),
        "KeyR" => Some(Key::KeyR),
        "KeyT" => Some(Key::KeyT),
        "KeyY" => Some(Key::KeyY),
        "KeyU" => Some(Key::KeyU),
        "KeyI" => Some(Key::KeyI),
        "KeyO" => Some(Key::KeyO),
        "KeyP" => Some(Key::KeyP),
        "LeftBracket" => Some(Key::LeftBracket),
        "RightBracket" => Some(Key::RightBracket),
        "KeyA" => Some(Key::KeyA),
        "KeyS" => Some(Key::KeyS),
        "KeyD" => Some(Key::KeyD),
        "KeyF" => Some(Key::KeyF),
        "KeyG" => Some(Key::KeyG),
        "KeyH" => Some(Key::KeyH),
        "KeyJ" => Some(Key::KeyJ),
        "KeyK" => Some(Key::KeyK),
        "KeyL" => Some(Key::KeyL),
        "SemiColon" => Some(Key::SemiColon),
        "Quote" => Some(Key::Quote),
        "BackSlash" => Some(Key::BackSlash),
        "IntlBackslash" => Some(Key::IntlBackslash),
        "KeyZ" => Some(Key::KeyZ),
        "KeyX" => Some(Key::KeyX),
        "KeyC" => Some(Key::KeyC),
        "KeyV" => Some(Key::KeyV),
        "KeyB" => Some(Key::KeyB),
        "KeyN" => Some(Key::KeyN),
        "KeyM" => Some(Key::KeyM),
        "Comma" => Some(Key::Comma),
        "Dot" => Some(Key::Dot),
        "Slash" => Some(Key::Slash),
        "Insert" => Some(Key::Insert),
        "KpReturn" => Some(Key::KpReturn),
        "KpMinus" => Some(Key::KpMinus),
        "KpPlus" => Some(Key::KpPlus),
        "KpMultiply" => Some(Key::KpMultiply),
        "KpDivide" => Some(Key::KpDivide),
        "Kp0" => Some(Key::Kp0),
        "Kp1" => Some(Key::Kp1),
        "Kp2" => Some(Key::Kp2),
        "Kp3" => Some(Key::Kp3),
        "Kp4" => Some(Key::Kp4),
        "Kp5" => Some(Key::Kp5),
        "Kp6" => Some(Key::Kp6),
        "Kp7" => Some(Key::Kp7),
        "Kp8" => Some(Key::Kp8),
        "Kp9" => Some(Key::Kp9),
        "KpDelete" => Some(Key::KpDelete),
        "Function" => Some(Key::Function),
        "VolumeUp" => Some(Key::VolumeUp),
        "VolumeDown" => Some(Key::VolumeDown),
        "VolumeMute" => Some(Key::VolumeMute),
        "BrightnessUp" => Some(Key::BrightnessUp),
        "BrightnessDown" => Some(Key::BrightnessDown),
        "PreviousTrack" => Some(Key::PreviousTrack),
        "PlayPause" => Some(Key::PlayPause),
        "PlayCd" => Some(Key::PlayCd),
        "NextTrack" => Some(Key::NextTrack),
        _ => None,
    }
}

/// Convert Button enum to string representation
pub fn button_to_string(button: &Button) -> String {
    match button {
        Button::Left => "Left",
        Button::Right => "Right",
        Button::Middle => "Middle",
        Button::Unknown(code) => return format!("Unknown({code})"),
    }
    .to_string()
}

/// Convert string to Button enum
pub fn string_to_button(s: &str) -> Option<Button> {
    match s {
        "Left" => Some(Button::Left),
        "Right" => Some(Button::Right),
        "Middle" => Some(Button::Middle),
        _ => None,
    }
}

/// Get all available keys
pub fn get_all_keys() -> Vec<Key> {
    vec![
        Key::Alt,
        Key::AltGr,
        Key::Backspace,
        Key::CapsLock,
        Key::ControlLeft,
        Key::ControlRight,
        Key::Delete,
        Key::DownArrow,
        Key::End,
        Key::Escape,
        Key::F1,
        Key::F2,
        Key::F3,
        Key::F4,
        Key::F5,
        Key::F6,
        Key::F7,
        Key::F8,
        Key::F9,
        Key::F10,
        Key::F11,
        Key::F12,
        Key::F13,
        Key::F14,
        Key::F15,
        Key::F16,
        Key::F17,
        Key::F18,
        Key::F19,
        Key::F20,
        Key::F21,
        Key::F22,
        Key::F23,
        Key::F24,
        Key::Home,
        Key::LeftArrow,
        Key::MetaLeft,
        Key::MetaRight,
        Key::PageDown,
        Key::PageUp,
        Key::Return,
        Key::RightArrow,
        Key::ShiftLeft,
        Key::ShiftRight,
        Key::Space,
        Key::Tab,
        Key::UpArrow,
        Key::PrintScreen,
        Key::ScrollLock,
        Key::Pause,
        Key::NumLock,
        Key::BackQuote,
        Key::Num1,
        Key::Num2,
        Key::Num3,
        Key::Num4,
        Key::Num5,
        Key::Num6,
        Key::Num7,
        Key::Num8,
        Key::Num9,
        Key::Num0,
        Key::Minus,
        Key::Equal,
        Key::KeyQ,
        Key::KeyW,
        Key::KeyE,
        Key::KeyR,
        Key::KeyT,
        Key::KeyY,
        Key::KeyU,
        Key::KeyI,
        Key::KeyO,
        Key::KeyP,
        Key::LeftBracket,
        Key::RightBracket,
        Key::KeyA,
        Key::KeyS,
        Key::KeyD,
        Key::KeyF,
        Key::KeyG,
        Key::KeyH,
        Key::KeyJ,
        Key::KeyK,
        Key::KeyL,
        Key::SemiColon,
        Key::Quote,
        Key::BackSlash,
        Key::IntlBackslash,
        Key::KeyZ,
        Key::KeyX,
        Key::KeyC,
        Key::KeyV,
        Key::KeyB,
        Key::KeyN,
        Key::KeyM,
        Key::Comma,
        Key::Dot,
        Key::Slash,
        Key::Insert,
        Key::KpReturn,
        Key::KpMinus,
        Key::KpPlus,
        Key::KpMultiply,
        Key::KpDivide,
        Key::Kp0,
        Key::Kp1,
        Key::Kp2,
        Key::Kp3,
        Key::Kp4,
        Key::Kp5,
        Key::Kp6,
        Key::Kp7,
        Key::Kp8,
        Key::Kp9,
        Key::KpDelete,
        Key::VolumeUp,
        Key::VolumeDown,
        Key::VolumeMute,
        Key::BrightnessUp,
        Key::BrightnessDown,
        Key::PreviousTrack,
        Key::PlayPause,
        Key::PlayCd,
        Key::NextTrack,
    ]
}

/// Get all available mouse buttons
pub fn get_all_buttons() -> Vec<Button> {
    vec![Button::Left, Button::Middle, Button::Right]
}
