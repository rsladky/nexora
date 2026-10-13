//! Reconnaissance vocale.
//!
//! Pipeline cible : micro (cpal) -> VAD -> `whisper-rs` (FR) -> `parse_intent`.
//! Cette version contient le trait de reconnaissance et le parser d'intentions (FR).

use nexora_core::VoiceIntent;

#[derive(Debug, thiserror::Error)]
pub enum VoiceError {
    #[error("micro indisponible : {0}")]
    Microphone(String),
    #[error("modèle introuvable ou invalide : {0}")]
    Model(String),
}

/// Transforme un segment audio (PCM mono 16 kHz) en texte.
pub trait SpeechRecognizer: Send {
    fn transcribe(&mut self, pcm_16khz: &[f32]) -> Result<String, VoiceError>;
}

/// Minuscules, sans accents ni ponctuation, espaces normalisés.
fn normalize(s: &str) -> String {
    let folded: String = s
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            c if c.is_alphanumeric() => c,
            _ => ' ',
        })
        .collect();
    folded.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Convertit une transcription française en intention. Préfixe « nexora » optionnel.
pub fn parse_intent(transcript: &str) -> Option<VoiceIntent> {
    let t = normalize(transcript);
    let t = t.strip_prefix("nexora").map(str::trim).unwrap_or(&t);

    if let Some(rest) = t.strip_prefix("tape ").or_else(|| t.strip_prefix("ecris ")) {
        return Some(VoiceIntent::Type(rest.to_string()));
    }

    use VoiceIntent::*;
    Some(match t {
        "copie" | "copier" => Copy,
        "colle" | "coller" => Paste,
        "coupe" | "couper" => Cut,
        "annule" | "annuler" => Undo,
        "selectionne tout" | "tout selectionner" => SelectAll,
        "nouvel onglet" => NewTab,
        "ferme" | "ferme l onglet" | "ferme la fenetre" => CloseTab,
        "defile vers le haut" | "monte" => ScrollUp,
        "defile vers le bas" | "descends" => ScrollDown,
        "clic droit" | "clique droit" => RightClick,
        "capture d ecran" | "screenshot" => Screenshot,
        "ouvre le navigateur" => OpenBrowser,
        "volume plus" | "monte le volume" => VolumeUp,
        "volume moins" | "baisse le volume" => VolumeDown,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_commands_with_accents_and_prefix() {
        assert_eq!(parse_intent("Copie."), Some(VoiceIntent::Copy));
        assert_eq!(
            parse_intent("Nexora, sélectionne tout"),
            Some(VoiceIntent::SelectAll)
        );
        assert_eq!(
            parse_intent("capture d'écran"),
            Some(VoiceIntent::Screenshot)
        );
    }

    #[test]
    fn parses_dictation() {
        assert_eq!(
            parse_intent("tape bonjour tout le monde"),
            Some(VoiceIntent::Type("bonjour tout le monde".into()))
        );
    }

    #[test]
    fn ignores_unknown() {
        assert_eq!(parse_intent("il fait beau"), None);
    }
}
