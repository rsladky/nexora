# Nexora — bootstrap du repo de l'app desktop

## Task
Organiser et démarrer le repo de l'application desktop Nexora (projet YDAYS 2026-2027, semaine C) : contrôle de l'ordinateur par gestes de la main (webcam) et commandes vocales. Pas le site vitrine.

## Context
- Source : `~/Downloads/Projet YDAYS - Nexora.pdf`. Objectif MVP : **≥ 5 gestes + ≥ 10 commandes vocales**, interface intuitive, commandes personnalisables, tests utilisateurs.
- Équipe prévue : 3 Data IA (toi = Data IA), 2 Dev, 1 Infra, 1 Marketing. Chef de projet : Jérémy CONTI.
- Choix validés : **Tauri v2 + backend Rust**, **IA 100 % Rust** (ONNX Runtime via `ort` pour les mains, `whisper-rs` pour la voix), frontend **React + TS** (Vite, bun), cibles **Windows + Linux**. Python uniquement dans `ml/` pour collecter/entraîner/exporter en ONNX.
- `/home/kinkin/Etude/Nexora` est vide, pas de git. Machine : Arch **aarch64**, **Wayland**, rust 1.99, bun, uv, clang présents ; **manquent `cmake` et `webkit2gtk-4.1`** (à installer : `sudo pacman -S cmake webkit2gtk-4.1`).
- Calendrier semaine C : YDAYS1 staffing 14/10/2026 · finalisation équipes 25/11 · **oral de validation 16/12** · prod 20/01, 21/01, 03/03, 24/03, 14/04, 05/05/2027 · recette 26/05 · **oral final 27/05/2027**.

## Architecture
```
Webcam ─► nexora-vision (nokhwa → palm detect + 21 landmarks ONNX → One-Euro filter → classifieur geste)
                                   │ GestureEvent
Micro ──► nexora-voice (cpal → VAD → whisper-rs FR → parser d'intentions)        ─► tokio mpsc ─► nexora-core::Mapper (config utilisateur)
                                   │ VoiceEvent                                                      │ Action
                                                                                       nexora-input (souris/clavier/raccourcis)
Tauri commands/events ◄──► UI React (statut, aperçu landmarks, éditeur de mappings, calibration)
```
Input : trait `InputBackend` avec `enigo` (Windows / X11) et `evdev`/uinput (Linux Wayland — nécessite règle udev, documentée).

## Files to create
```
Nexora/
├── Cargo.toml                 # workspace (crates/*, app/src-tauri)
├── crates/
│   ├── nexora-core/           # types Gesture, VoiceIntent, Action ; Mapper ; config serde (TOML) ; tests
│   ├── nexora-vision/         # capture caméra, inférence ort, filtre One-Euro, classifieur (règles puis MLP ONNX)
│   ├── nexora-voice/          # capture audio, VAD énergie, whisper-rs, grammaire FR → VoiceIntent ; tests parser
│   └── nexora-input/          # trait InputBackend + backends enigo / uinput
├── app/                       # `bun create tauri-app` React-TS
│   ├── src/                   # pages : Dashboard, Gestes, Commandes vocales, Réglages
│   └── src-tauri/             # commandes start/stop engine, get/set config ; events vers l'UI
├── ml/                        # projet uv : collect_landmarks.py, train_gesture.py, export_onnx.py, notebooks/
├── models/                    # ignoré par git ; rempli par scripts/download-models.sh (+ .ps1)
├── scripts/                   # download-models.sh/.ps1, setup-linux.sh (udev uinput)
├── docs/
│   ├── ARCHITECTURE.md
│   ├── GESTURES.md            # catalogue MVP
│   ├── VOICE_COMMANDS.md      # catalogue MVP
│   ├── ROADMAP.md             # aligné dates YDAYS semaine C + répartition par rôle
│   └── CONTRIBUTING.md        # conventions git (branches, PR, commits)
├── .agents/plans/NEXORA-DESKTOP-BOOTSTRAP.md   # copie de ce plan
├── .github/workflows/ci.yml   # fmt, clippy, test (ubuntu + windows), bun build
├── README.md · .gitignore · rust-toolchain.toml · .editorconfig
```

## Catalogue MVP (proposé, éditable)
**Gestes (5)** : index levé → déplacer curseur · pincement pouce-index → clic gauche · pincement pouce-majeur → clic droit · deux doigts haut/bas → scroll · poing fermé maintenu → glisser-déposer · paume ouverte 1 s → pause/reprise du suivi.
**Voix (10+)**, FR, préfixe optionnel « Nexora » : copie, colle, coupe, annule, sélectionne tout, nouvel onglet, ferme la fenêtre/l'onglet, défile vers le haut/bas, clic droit, capture d'écran, ouvre le navigateur, volume plus/moins, « tape … » (dictée).

## Steps
1. `git init`, `.gitignore` (target/, node_modules/, models/, ml/.venv, datasets), README, rust-toolchain.toml, copie du plan dans `.agents/plans/`.
2. Workspace Cargo + 4 crates avec API minimale qui compile : types et `Mapper` (core) avec tests unitaires ; trait `InputBackend` ; stubs vision/voice derrière des traits (`HandTracker`, `SpeechRecognizer`) pour pouvoir mocker.
3. Scaffold Tauri v2 React-TS avec `bunx create-tauri-app` dans `app/`, brancher les crates, commandes `engine_start/stop`, `config_get/set`, event `nexora://status`. UI minimale (statut + toggle).
4. `scripts/download-models.sh` : modèles mains ONNX (palm detection + hand landmark MediaPipe, PINTO model zoo) et `ggml-base` (ou `small`) Whisper multilingue.
5. POC vertical : caméra → landmarks → curseur suit l'index + pincement = clic ; push-to-talk → whisper → « copie »/« colle ».
6. `ml/` : projet uv (mediapipe, opencv, torch/sklearn, onnx) avec script de collecte de landmarks étiquetés et entraînement d'un petit MLP exporté en ONNX (remplacera le classifieur à règles).
7. Docs (ARCHITECTURE, GESTURES, VOICE_COMMANDS, ROADMAP par séance YDAYS et par rôle, CONTRIBUTING) + CI GitHub Actions.

## Roadmap (résumé pour docs/ROADMAP.md)
| Date | Jalon |
|---|---|
| 14/10 → 25/11 | Staffing, repo prêt, POC gestes+voix, catalogue validé |
| 16/12 | Oral de validation : démo POC + archi + maquette UI |
| 20–21/01 | Dev gestes & voix (classifieur ML, grammaire voix) |
| 03/03 | Intégration UI + personnalisation des mappings |
| 24/03 | Tests fonctionnels + premiers tests utilisateurs |
| 14/04 | Optimisation (latence, CPU, faux positifs) |
| 05/05 | Finalisation, packaging Windows (.msi) + Linux (AppImage) |
| 26–27/05 | Recette, rapport de tests, oral final |

## Conventions
- Rust : `cargo fmt`, `clippy -D warnings`, erreurs `thiserror`/`anyhow`, logs `tracing`.
- JS : bun uniquement (pas npm), TS strict, ESLint/Prettier.
- Git : `main` protégée, branches `feat/…`, `fix/…`, PR relue, commits conventionnels.
- Modèles et datasets jamais commités (script de téléchargement).

## Verification
- `cargo fmt --check && cargo clippy --workspace && cargo test --workspace` (tests Mapper + parser d'intentions).
- `cd app && bun install && bun run tauri dev` : la fenêtre s'ouvre, statut affiché.
- POC : lancer l'engine, l'index déplace le curseur, pincement clique ; dire « copie » / « colle » fonctionne dans un éditeur (tester sur Wayland via uinput).
- CI verte sur ubuntu + windows.
