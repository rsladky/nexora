//! Démo caméra réelle : webcam -> suivi ONNX -> moteur -> actions journalisées (aucune action système).
//! `cargo run --release -p nexora-engine --features live --example live -- [index] [secondes]`

use std::time::{Duration, Instant};

use nexora_core::Config;
use nexora_engine::Engine;
use nexora_input::LogBackend;
use nexora_vision::hand::CameraHandTracker;
use nexora_vision::HandTracker;

fn main() {
    let mut args = std::env::args().skip(1);
    let index = args.next().and_then(|a| a.parse().ok());
    let secs: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(30);

    let mut cam = CameraHandTracker::open(index, None).expect("caméra + modèles");
    let mut engine = Engine::new(Config::default(), LogBackend::default());
    println!("caméra ouverte, {secs} s : levez l'index, pincez, fermez le poing…");

    let (end, mut frames, mut last_gesture) =
        (Instant::now() + Duration::from_secs(secs), 0u32, None);
    while Instant::now() < end {
        let hand = cam.next().expect("frame");
        frames += 1;
        let actions = engine.on_frame(hand.as_ref()).expect("moteur");
        for a in actions {
            println!("  action : {a:?}");
        }
        let gesture = engine.current_gesture();
        if gesture != last_gesture {
            println!("geste : {gesture:?}");
            last_gesture = gesture;
        }
    }
    println!("{frames} frames en {secs} s");
}
