//! Exécution des `Action` sur le système.
//!
//! Backends prévus : `enigo` (Windows, X11) et `uinput` (Linux/Wayland).
//! Ils seront ajoutés derrière des features Cargo ; pour l'instant seul `LogBackend` existe.

use nexora_core::Action;

#[derive(Debug, thiserror::Error)]
pub enum InputError {
    #[error("backend d'entrée indisponible : {0}")]
    Unavailable(String),
    #[error("action non supportée par ce backend : {0:?}")]
    Unsupported(Action),
}

pub trait InputBackend: Send {
    fn execute(&mut self, action: &Action) -> Result<(), InputError>;
}

/// Backend de test/dev : journalise les actions sans toucher au système.
#[derive(Default)]
pub struct LogBackend {
    pub executed: Vec<Action>,
}

impl InputBackend for LogBackend {
    fn execute(&mut self, action: &Action) -> Result<(), InputError> {
        tracing::info!(?action, "action");
        self.executed.push(action.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_backend_records_actions() {
        let mut b = LogBackend::default();
        b.execute(&Action::TogglePause).unwrap();
        assert_eq!(b.executed, vec![Action::TogglePause]);
    }
}
