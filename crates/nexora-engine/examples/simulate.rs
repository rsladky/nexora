//! Simulation sans matériel : rejoue un scénario de gestes, puis lit des transcriptions
//! sur stdin. `cargo run -p nexora-engine --example simulate`

use std::io::BufRead;

use nexora_core::Config;
use nexora_engine::Engine;
use nexora_input::LogBackend;
use nexora_vision::synthetic::{hand, pinch, pointing_at};
use nexora_vision::{HandTracker, Landmarks, ScriptedTracker};

fn scenario() -> Vec<(&'static str, Option<Landmarks>)> {
    let mut frames = Vec::new();
    let mut add = |label, frame: Option<Landmarks>, n| {
        for _ in 0..n {
            frames.push((label, frame));
        }
    };
    for i in 0..4 {
        add(
            "index levé",
            Some(pointing_at(0.3 + 0.1 * i as f32, 0.2)),
            1,
        );
    }
    add("pincement", Some(pinch()), 4);
    add("poing", Some(hand([false; 4])), 4);
    add("main retirée", None, 3);
    add("paume (pause)", Some(hand([true; 4])), 4);
    add("main retirée", None, 3);
    add("paume (reprise)", Some(hand([true; 4])), 4);
    frames
}

fn main() {
    let mut engine = Engine::new(Config::default(), LogBackend::default());
    let script = scenario();
    let labels: Vec<_> = script.iter().map(|(l, _)| *l).collect();
    let mut cam = ScriptedTracker::new(script.into_iter().map(|(_, f)| f));

    println!("— Gestes —");
    for label in labels {
        let frame = cam.next().expect("mock caméra");
        for a in engine.on_frame(frame.as_ref()).expect("backend") {
            println!("[{label}] -> {a:?}");
        }
    }

    println!("— Voix : tape une phrase (Ctrl+D pour quitter) —");
    for line in std::io::stdin().lock().lines() {
        let line = line.expect("stdin");
        match engine.on_transcript(&line).expect("backend").as_slice() {
            [] => println!("(non reconnu ou en pause)"),
            actions => actions.iter().for_each(|a| println!("-> {a:?}")),
        }
    }
}
