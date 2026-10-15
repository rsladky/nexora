# Viewer de détections (POC) : frame + rectangle de paume + 21 points dans l'app Tauri

## Task
Afficher dans l'app Nexora (Tauri + React) le flux webcam, avec par-dessus :
- le rectangle de détection de la paume, rotaté, avec ses 7 points-clés ;
- les 21 landmarks reliés en squelette ;
- le geste reconnu et les actions déclenchées.

Pour le POC du 16/12, les détections viennent d'un process Python séparé (sidecar MediaPipe). Le portage `ort` le remplacera plus tard.

## Context
- **Pas encore d'app.** `app/` n'existe pas : il faut scaffolder Tauri 2 + React (`bunx create-tauri-app`, CLI via `bunx @tauri-apps/cli`). Le workspace Cargo ne contient que `crates/*`.
- **Le moteur est prêt.** `nexora-engine::Engine::on_frame(Option<&Landmarks>)` existe déjà (`crates/nexora-engine/src/lib.rs`). On l'utilise avec `LogBackend`, le seul backend d'entrée disponible (`crates/nexora-input/src/lib.rs`), donc aucune action réelle sur le système. Le viewer affiche les actions au lieu de les exécuter.
- **Le rectangle de paume n'est pas fourni par MediaPipe Python.** Ni les Tasks (`HandLandmarker`) ni l'ancienne API `solutions.hands` n'exposent la détection de paume : seulement les landmarks et la latéralité. Pour l'afficher, le sidecar lance **lui-même** `hand_detector` en ONNX (via `onnxruntime` + numpy) : ancres, décodage, NMS pondérée.
  - C'est exactement la logique à porter ensuite en Rust. Ce code Python devient donc l'**implémentation de référence** qui servira aux tests de comparaison du portage `ort`.
- **Python** : la machine a Python 3.14 et les wheels `mediapipe` aarch64 ne suivent pas toujours les dernières versions. Il faut épingler Python 3.12 via `uv` (`ml/.python-version`).
- **Caméra Asahi** : `/dev/video1`, en NV12. OpenCV (`CAP_V4L2`) doit gérer la conversion. Sinon, plan B avec un pipeline GStreamer `v4l2src device=/dev/video1 ! videoconvert ! appsink`. L'index de la caméra doit être un argument.

## Protocole sidecar → Rust
Le sidecar écrit sur stdout une ligne JSON par frame (JSON Lines). Le JPEG est en base64, environ 30 Ko en 640×480, ce qui suffit pour un POC à 30 fps.
```json
{"t_ms":123,"w":640,"h":480,"jpeg":"<base64>",
 "palms":[{"score":0.93,"cx":0.5,"cy":0.6,"w":0.2,"h":0.2,"rotation":0.12,"keypoints":[[x,y],...7]}],
 "hands":[{"handedness":"Right","score":0.98,"landmarks":[[x,y,z],...21]}]}
```
Toutes les coordonnées sont normalisées 0..1 dans le repère de l'image **non miroir**.

## Files to modify / create
- **`ml/sidecar/hand_sidecar.py`** (nouveau)
  - Args : `--camera 1 --width 640 --height 480 --models ../models`.
  - Boucle : capture OpenCV → MediaPipe `HandLandmarker` (mode VIDEO) → `palm.py` → JSON sur stdout.
  - Logs sur stderr uniquement.
- **`ml/sidecar/palm.py`** (nouveau) : détecteur de paume de référence.
  - Letterbox 192×192.
  - Génération des 2016 ancres (24×24×2 + 12×12×6).
  - Sigmoïde, décodage, NMS pondérée (IoU 0,3), retrait du letterbox.
  - Calcul de la rotation à partir des points-clés 0 et 2.
- **`ml/scripts/fetch_models.py`** (nouveau)
  1. Télécharge `hand_landmarker.task`.
  2. Extrait `hand_detector.tflite`.
  3. Le convertit en `models/palm_detection.onnx` avec `tf2onnx`.
  - Les modèles ne sont pas commités : ajouter `models/` au `.gitignore`.
- **`ml/pyproject.toml`** : ajouter `onnxruntime` et `tf2onnx`, puis épingler Python 3.12.
- **`crates/nexora-vision/src/sidecar.rs`** (nouveau, feature `sidecar = ["dep:serde", "dep:serde_json", "dep:base64"]`)
  - Types sérialisables : `VisionFrame { t_ms, width, height, jpeg: Vec<u8>, palms: Vec<PalmDetection>, hands: Vec<HandDetection> }`, `PalmDetection { score, cx, cy, w, h, rotation, keypoints: [[f32;2];7] }` et `HandDetection { handedness, score, landmarks: Landmarks }`.
  - `SidecarTracker::spawn(cmd) -> Result<Self, VisionError>` : lance le process et lit stdout dans un thread qui envoie les messages sur un canal.
  - `next_frame() -> Result<Option<VisionFrame>, VisionError>`.
  - Le trait `HandTracker` est implémenté avec la première main → `Option<Landmarks>`. Le moteur reste ainsi inchangé.
  - Ajouter une variante `VisionError::Sidecar(String)`.
- **`crates/nexora-engine/src/lib.rs`** : ajouter `pub fn current_gesture(&self) -> Option<Gesture>`, qui délègue à `self.tracker.current()`, pour l'afficher dans le viewer.
- **`app/`** (nouveau, scaffold Tauri 2 + React + TS + Vite, avec bun)
  - **`app/src-tauri/src/vision.rs`**
    - Commandes `start_vision(camera: u32)` / `stop_vision()`.
    - Un thread : `SidecarTracker` → `Engine<LogBackend>::on_frame` → émission de l'événement `vision://frame` avec le payload `{ jpeg_b64, w, h, palms, hands, gesture, actions, fps }`.
    - L'arrêt tue le process sidecar, à la fermeture de la fenêtre aussi.
  - **`app/src/components/DetectionViewer.tsx`**
    - `<canvas>` : `drawImage` de la frame décodée, puis les calques :
      - le rectangle de paume rotaté (`ctx.translate` / `rotate`) avec ses 7 points-clés et son score ;
      - les 21 points et les connexions du squelette (constante `HAND_CONNECTIONS` à 21 arêtes), colorés par doigt ;
      - un bandeau avec le geste courant, la latéralité, les FPS et les dernières actions.
    - Cases à cocher pour chaque calque (paume, landmarks, squelette), plus une bascule « miroir », appliquée avec `scaleX(-1)` sur le canvas entier sauf le texte.
  - `app/src/App.tsx` : page unique avec boutons Démarrer/Arrêter, choix de l'index caméra et le viewer.
- **`Cargo.toml`** (racine) : ajouter `app/src-tauri` aux `members`, et `serde_json` + `base64` aux dépendances du workspace.
- **`docs/ARCHITECTURE.md`, `README.md`** : décrire le sidecar, son protocole et les commandes de lancement.

## Steps
0. Copier ce plan dans `.agents/plans/DETECTION-VIEWER-POC.md`. Partir d'une branche dédiée `feat/detection-viewer-poc`, idéalement dans un worktree séparé de la branche du spike caméra.
1. **Python seul** : `fetch_models.py`, puis `hand_sidecar.py` avec MediaPipe uniquement. Vérifier `uv run python sidecar/hand_sidecar.py --camera 1 | head -c 500`.
2. Ajouter `palm.py`. Pour valider, comparer le centre de la paume détectée au centre des landmarks MediaPipe : ils doivent être proches.
3. Écrire `SidecarTracker` en Rust, avec un test unitaire qui parse une ligne JSON d'exemple (pas de matériel).
4. Scaffolder `app/`, ajouter la commande `start_vision` et l'événement `vision://frame`.
5. Écrire `DetectionViewer` : d'abord la frame, puis les landmarks, puis le rectangle de paume rotaté, puis le bandeau.
6. Mettre à jour la doc, lancer `fmt`/`clippy`/tests, puis commiter en `feat(vision): ...` et `feat(app): ...`.

## Conventions
- Rust : commentaires en français, `thiserror`, code matériel ou process derrière une feature désactivée par défaut, pour que la CI reste sans dépendance.
- Python : `uv`, scripts dans `ml/`.
- Front : bun (`bun install`, `bun run tauri dev`).
- Le moteur garde `LogBackend` dans le viewer. Aucune action système pendant le POC du viewer.

## Risques
- **Wheels MediaPipe aarch64** : si l'installation échoue en Python 3.12, essayer 3.11. Dernier recours : faire tourner aussi le modèle de landmarks via `onnxruntime` dans le sidecar, avec un ROI calculé en Python. Ça prend plus de temps, mais c'est la même logique que le portage Rust.
- **NV12 sous OpenCV** : utiliser le plan B GStreamer.
- **Débit des événements Tauri** (base64 à 30 fps) : si ça saccade, plafonner à 15 fps côté émission, ou servir la frame via un protocole custom `nexora://frame`.

## Verification
- `cargo test --workspace` reste vert. Il inclut le test de parsing du JSON du sidecar avec `--features sidecar`.
- `cd ml && uv run python sidecar/hand_sidecar.py --camera 1` doit sortir des lignes JSON avec `palms` et `hands` non vides quand une main est visible.
- `cd app && bun run tauri dev`, puis Démarrer :
  - le flux s'affiche à environ 25–30 fps ;
  - le rectangle de paume suit et tourne avec la main ;
  - les 21 points se superposent correctement aux doigts, y compris en mode miroir ;
  - le bandeau affiche `point` / `pinch_index` et les actions `MoveCursor` / `Click` journalisées.
- Arrêter, puis vérifier qu'aucun process `hand_sidecar.py` ne reste vivant (`pgrep -f hand_sidecar`).
