use serde::{Deserialize, Serialize};

/// Gestes de la main reconnus par `nexora-vision`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gesture {
    /// Index levé : le curseur suit le bout de l'index.
    Point,
    /// Pincement pouce-index.
    PinchIndex,
    /// Pincement pouce-majeur.
    PinchMiddle,
    /// Deux doigts levés (scroll selon le mouvement vertical).
    TwoFingers,
    /// Poing fermé (glisser-déposer tant qu'il est maintenu).
    Fist,
    /// Paume ouverte (pause / reprise du suivi).
    OpenPalm,
}

/// Intentions vocales produites par le parser de `nexora-voice`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceIntent {
    Copy,
    Paste,
    Cut,
    Undo,
    SelectAll,
    NewTab,
    CloseTab,
    ScrollUp,
    ScrollDown,
    RightClick,
    Screenshot,
    OpenBrowser,
    VolumeUp,
    VolumeDown,
    /// Dictée : tape le texte tel quel.
    Type(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Key {
    Ctrl,
    Shift,
    Alt,
    Meta,
    Tab,
    Enter,
    Escape,
    Char(char),
    VolumeUp,
    VolumeDown,
    PrintScreen,
}

/// Action système à exécuter par un `InputBackend`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    /// Déplacement absolu normalisé (0.0..=1.0 sur l'écran).
    MoveCursor {
        x: f32,
        y: f32,
    },
    Click {
        button: MouseButton,
    },
    ButtonDown {
        button: MouseButton,
    },
    ButtonUp {
        button: MouseButton,
    },
    Scroll {
        dy: i32,
    },
    /// Raccourci : touches maintenues puis relâchées dans l'ordre inverse.
    Shortcut {
        keys: Vec<Key>,
    },
    TypeText {
        text: String,
    },
    /// Lance une commande (ex. navigateur).
    Launch {
        command: String,
    },
    TogglePause,
}

/// Événement entrant dans le `Mapper`.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    GestureStart(Gesture),
    GestureEnd(Gesture),
    Voice(VoiceIntent),
}
