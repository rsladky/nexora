use crate::{Action, Config, Event, VoiceIntent};

/// Traduit les événements (gestes/voix) en actions selon la configuration.
/// Gère aussi l'état de pause : seules les actions `TogglePause` passent en pause.
pub struct Mapper {
    config: Config,
    paused: bool,
}

impl Mapper {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            paused: false,
        }
    }

    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Retourne l'action à exécuter (si elle existe et n'est pas bloquée par la pause).
    pub fn handle(&mut self, event: Event) -> Option<Action> {
        let action = match event {
            Event::GestureStart(g) => self.config.gestures.get(&g).cloned(),
            // Relâcher un geste ne fait que terminer un glisser (ButtonDown -> ButtonUp).
            Event::GestureEnd(g) => match self.config.gestures.get(&g) {
                Some(Action::ButtonDown { button }) => Some(Action::ButtonUp { button: *button }),
                _ => None,
            },
            Event::Voice(VoiceIntent::Type(text)) => Some(Action::TypeText { text }),
            Event::Voice(intent) => self.config.voice.get(&intent).cloned(),
        }?;

        if matches!(action, Action::TogglePause) {
            self.paused = !self.paused;
            return Some(action);
        }
        if self.paused {
            return None;
        }
        Some(action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Gesture, MouseButton};

    #[test]
    fn pinch_clicks() {
        let mut m = Mapper::new(Config::default());
        assert_eq!(
            m.handle(Event::GestureStart(Gesture::PinchIndex)),
            Some(Action::Click {
                button: MouseButton::Left
            })
        );
    }

    #[test]
    fn fist_drags_then_releases() {
        let mut m = Mapper::new(Config::default());
        assert!(matches!(
            m.handle(Event::GestureStart(Gesture::Fist)),
            Some(Action::ButtonDown { .. })
        ));
        assert!(matches!(
            m.handle(Event::GestureEnd(Gesture::Fist)),
            Some(Action::ButtonUp { .. })
        ));
    }

    #[test]
    fn pause_blocks_actions_until_toggled_back() {
        let mut m = Mapper::new(Config::default());
        m.handle(Event::GestureStart(Gesture::OpenPalm));
        assert!(m.is_paused());
        assert_eq!(m.handle(Event::Voice(VoiceIntent::Copy)), None);
        m.handle(Event::GestureStart(Gesture::OpenPalm));
        assert!(!m.is_paused());
        assert!(m.handle(Event::Voice(VoiceIntent::Copy)).is_some());
    }

    #[test]
    fn dictation_types_text() {
        let mut m = Mapper::new(Config::default());
        assert_eq!(
            m.handle(Event::Voice(VoiceIntent::Type("bonjour".into()))),
            Some(Action::TypeText {
                text: "bonjour".into()
            })
        );
    }
}
