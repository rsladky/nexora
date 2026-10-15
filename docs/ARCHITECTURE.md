# Architecture

```
Webcam ─► nexora-vision  ─ GestureEvent ─┐
                                         ├─► Mapper (nexora-core, config TOML) ─ Action ─► nexora-input
Micro  ─► nexora-voice   ─ VoiceIntent ──┘
                                                       ▲
                          Tauri commands/events ◄──► UI React
```

## Crates

- **nexora-core** : `Gesture`, `VoiceIntent`, `Action`, `Event`, `Config` (TOML), `Mapper`. Aucun I/O : 100 % testable.
- **nexora-vision** : trait `HandTracker` (caméra+ONNX ou mock), `classify()` à règles. Feature `camera` (désactivée par défaut) : capture webcam RGB (`v4l` sous Linux, `nokhwa` ailleurs), sonde `cargo run -p nexora-vision --features camera --example camera_probe`. Module `hand` : détecteur de paume + landmarks MediaPipe (ROI rotatée, suivi sans re-détection, filtre One-Euro) ; logique pure testée en CI, inférence `ort` derrière la feature `onnx` (modèles via `ml/scripts/fetch_models.py`, dossier `models/` ou `NEXORA_MODELS_DIR`). Sonde : `cargo run --release -p nexora-vision --features camera,onnx --example hand_probe -- 1 300 --serve` (flux MJPEG annoté sur `http://127.0.0.1:8080`). À venir : MLP ONNX.
- **nexora-voice** : trait `SpeechRecognizer`, `parse_intent()` FR. À venir : `cpal`, VAD, `whisper-rs`.
- **nexora-engine** : `Engine<B: InputBackend>` relie `classify` + `GestureTracker` (debounce) + `parse_intent` + `Mapper` + backend. `on_frame()` / `on_transcript()`. Démo sans matériel : `cargo run -p nexora-engine --example simulate` ; démo caméra réelle : `cargo run --release -p nexora-engine --features live --example live`.
- **nexora-input** : trait `InputBackend`, `LogBackend`. À venir : backends `enigo` et `uinput` (features Cargo).

## Choix

- **Tout Rust à l'exécution** : un seul binaire, pas de runtime Python à packager. Python ne sert qu'à entraîner/exporter (`ml/`).
- **Traits + mocks** : chaque couche matérielle (caméra, micro, système) est derrière un trait pour tester sans matériel.
- **Wayland** : le contrôle souris/clavier passe par `uinput` (règle udev requise) ; X11 et Windows par `enigo`.
- **Vie privée** : tout est traité en local, aucune donnée caméra/audio ne quitte la machine.

## Flux Tauri (à implémenter)

Commandes : `engine_start`, `engine_stop`, `config_get`, `config_set`. Événements : `nexora://status`, `nexora://gesture`, `nexora://voice`.

## Caméra sur Asahi Linux (Mac Apple Silicon)

La webcam est `/dev/video1` (driver `apple-isp`, NV12 uniquement) ; `/dev/video0` est le décodeur vidéo matériel `avd`, pas une caméra. `nokhwa` échoue sur ce pilote (intervalle de trames `256/7680` non réduit), d'où le backend `v4l` direct sous Linux. La sélection automatique ignore `avd`.
