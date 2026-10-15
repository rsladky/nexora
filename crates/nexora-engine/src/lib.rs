//! Moteur : relie vision/voix -> `Mapper` -> `InputBackend`.
//!
//! Aucun I/O matériel ici : les frames et les transcriptions sont poussées par l'appelant
//! (caméra/micro réels, mocks de test ou commandes Tauri).

use nexora_core::{Action, Config, Event, Gesture, Mapper};
use nexora_input::{InputBackend, InputError};
use nexora_vision::{classify, idx, GestureTracker, Landmarks};
use nexora_voice::parse_intent;

pub struct Engine<B: InputBackend> {
    mapper: Mapper,
    tracker: GestureTracker,
    backend: B,
}

impl<B: InputBackend> Engine<B> {
    pub fn new(config: Config, backend: B) -> Self {
        Self {
            mapper: Mapper::new(config),
            tracker: GestureTracker::default(),
            backend,
        }
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Geste actuellement validé (après debounce).
    pub fn current_gesture(&self) -> Option<Gesture> {
        self.tracker.current()
    }

    pub fn is_paused(&self) -> bool {
        self.mapper.is_paused()
    }

    /// Traite une frame (`None` = aucune main). Retourne les actions exécutées.
    pub fn on_frame(&mut self, landmarks: Option<&Landmarks>) -> Result<Vec<Action>, InputError> {
        let raw = landmarks.and_then(classify);
        let mut actions: Vec<Action> = self
            .tracker
            .update(raw)
            .into_iter()
            .filter_map(|e| self.mapper.handle(e))
            .collect();

        // `Point` : le curseur suit le bout de l'index (image en miroir).
        if let (Some(Gesture::Point), Some(l), false) =
            (self.tracker.current(), landmarks, self.mapper.is_paused())
        {
            let tip = l[idx::INDEX_TIP];
            actions.push(Action::MoveCursor {
                x: (1.0 - tip[0]).clamp(0.0, 1.0),
                y: tip[1].clamp(0.0, 1.0),
            });
        }

        self.run(actions)
    }

    /// Traite une transcription vocale. Liste vide si non reconnue ou en pause.
    pub fn on_transcript(&mut self, text: &str) -> Result<Vec<Action>, InputError> {
        let actions = parse_intent(text)
            .and_then(|intent| self.mapper.handle(Event::Voice(intent)))
            .into_iter()
            .collect();
        self.run(actions)
    }

    fn run(&mut self, actions: Vec<Action>) -> Result<Vec<Action>, InputError> {
        for a in &actions {
            self.backend.execute(a)?;
        }
        Ok(actions)
    }
}
