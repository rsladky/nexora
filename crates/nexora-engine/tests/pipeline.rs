use nexora_core::{Action, Config, MouseButton};
use nexora_engine::Engine;
use nexora_input::LogBackend;
use nexora_vision::synthetic::{hand, pinch, pointing_at};

/// Pousse une même frame `n` fois (le debounce exige 3 frames identiques).
fn feed(e: &mut Engine<LogBackend>, frame: Option<&nexora_vision::Landmarks>, n: usize) {
    for _ in 0..n {
        e.on_frame(frame).unwrap();
    }
}

#[test]
fn gestures_and_voice_drive_the_backend() {
    let mut e = Engine::new(Config::default(), LogBackend::default());

    // Point : le curseur suit l'index (miroir horizontal) dès que le geste est validé.
    feed(&mut e, Some(&pointing_at(0.25, 0.2)), 3);
    assert_eq!(
        e.backend().executed.last(),
        Some(&Action::MoveCursor { x: 0.75, y: 0.2 })
    );

    // Pincement : un seul clic gauche.
    feed(&mut e, Some(&pinch()), 5);
    let clicks = e
        .backend()
        .executed
        .iter()
        .filter(|a| {
            matches!(
                a,
                Action::Click {
                    button: MouseButton::Left
                }
            )
        })
        .count();
    assert_eq!(clicks, 1);

    // Poing : ButtonDown, puis ButtonUp au relâchement.
    feed(&mut e, Some(&hand([false; 4])), 3);
    feed(&mut e, None, 3);
    let log = &e.backend().executed;
    assert!(log.contains(&Action::ButtonDown {
        button: MouseButton::Left
    }));
    assert_eq!(
        log.last(),
        Some(&Action::ButtonUp {
            button: MouseButton::Left
        })
    );

    // Voix.
    assert_eq!(e.on_transcript("Nexora, copie").unwrap().len(), 1);
    assert!(e.on_transcript("il fait beau").unwrap().is_empty());

    // Paume : pause, la voix est bloquée ; une seconde paume reprend.
    feed(&mut e, Some(&hand([true; 4])), 3);
    assert!(e.is_paused());
    assert!(e.on_transcript("copie").unwrap().is_empty());
    feed(&mut e, None, 3);
    feed(&mut e, Some(&hand([true; 4])), 3);
    assert!(!e.is_paused());
    assert_eq!(e.on_transcript("colle").unwrap().len(), 1);
}
