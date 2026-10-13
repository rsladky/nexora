//! Types partagés, mapping événements -> actions et configuration utilisateur.

mod config;
mod mapper;
mod types;

pub use config::{Config, ConfigError};
pub use mapper::Mapper;
pub use types::{Action, Event, Gesture, Key, MouseButton, VoiceIntent};
