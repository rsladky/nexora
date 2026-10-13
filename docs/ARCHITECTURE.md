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
- **nexora-vision** : trait `HandTracker` (caméra+ONNX ou mock), `classify()` à règles. À venir : `nokhwa`, `ort`, filtre One-Euro, MLP ONNX.
- **nexora-voice** : trait `SpeechRecognizer`, `parse_intent()` FR. À venir : `cpal`, VAD, `whisper-rs`.
- **nexora-input** : trait `InputBackend`, `LogBackend`. À venir : backends `enigo` et `uinput` (features Cargo).

## Choix

- **Tout Rust à l'exécution** : un seul binaire, pas de runtime Python à packager. Python ne sert qu'à entraîner/exporter (`ml/`).
- **Traits + mocks** : chaque couche matérielle (caméra, micro, système) est derrière un trait pour tester sans matériel.
- **Wayland** : le contrôle souris/clavier passe par `uinput` (règle udev requise) ; X11 et Windows par `enigo`.
- **Vie privée** : tout est traité en local, aucune donnée caméra/audio ne quitte la machine.

## Flux Tauri (à implémenter)

Commandes : `engine_start`, `engine_stop`, `config_get`, `config_set`. Événements : `nexora://status`, `nexora://gesture`, `nexora://voice`.
