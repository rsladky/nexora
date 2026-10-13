use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Action, Gesture, Key, MouseButton, VoiceIntent};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("configuration invalide : {0}")]
    Parse(#[from] toml::de::Error),
    #[error("sérialisation impossible : {0}")]
    Serialize(#[from] toml::ser::Error),
}

/// Configuration utilisateur : mapping personnalisable gestes/voix -> actions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub gestures: HashMap<Gesture, Action>,
    pub voice: HashMap<VoiceIntent, Action>,
}

impl Config {
    pub fn from_toml(s: &str) -> Result<Self, ConfigError> {
        Ok(toml::from_str(s)?)
    }

    pub fn to_toml(&self) -> Result<String, ConfigError> {
        Ok(toml::to_string_pretty(self)?)
    }
}

fn shortcut(keys: &[Key]) -> Action {
    Action::Shortcut {
        keys: keys.to_vec(),
    }
}

impl Default for Config {
    fn default() -> Self {
        use Key::{Char, Ctrl};
        let gestures = HashMap::from([
            (
                Gesture::PinchIndex,
                Action::Click {
                    button: MouseButton::Left,
                },
            ),
            (
                Gesture::PinchMiddle,
                Action::Click {
                    button: MouseButton::Right,
                },
            ),
            (
                Gesture::Fist,
                Action::ButtonDown {
                    button: MouseButton::Left,
                },
            ),
            (Gesture::OpenPalm, Action::TogglePause),
        ]);
        let voice = HashMap::from([
            (VoiceIntent::Copy, shortcut(&[Ctrl, Char('c')])),
            (VoiceIntent::Paste, shortcut(&[Ctrl, Char('v')])),
            (VoiceIntent::Cut, shortcut(&[Ctrl, Char('x')])),
            (VoiceIntent::Undo, shortcut(&[Ctrl, Char('z')])),
            (VoiceIntent::SelectAll, shortcut(&[Ctrl, Char('a')])),
            (VoiceIntent::NewTab, shortcut(&[Ctrl, Char('t')])),
            (VoiceIntent::CloseTab, shortcut(&[Ctrl, Char('w')])),
            (VoiceIntent::ScrollUp, Action::Scroll { dy: 5 }),
            (VoiceIntent::ScrollDown, Action::Scroll { dy: -5 }),
            (
                VoiceIntent::RightClick,
                Action::Click {
                    button: MouseButton::Right,
                },
            ),
            (VoiceIntent::Screenshot, shortcut(&[Key::PrintScreen])),
            (
                VoiceIntent::OpenBrowser,
                Action::Launch {
                    command: "xdg-open https://".into(),
                },
            ),
            (VoiceIntent::VolumeUp, shortcut(&[Key::VolumeUp])),
            (VoiceIntent::VolumeDown, shortcut(&[Key::VolumeDown])),
        ]);
        Self { gestures, voice }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_covers_mvp_voice_commands() {
        assert!(Config::default().voice.len() >= 10);
    }

    #[test]
    fn toml_roundtrip() {
        let cfg = Config::default();
        let parsed = Config::from_toml(&cfg.to_toml().unwrap()).unwrap();
        assert_eq!(cfg, parsed);
    }
}
