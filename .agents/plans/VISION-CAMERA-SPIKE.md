# Spike webcam : capture caméra avec `nokhwa` dans nexora-vision

## Task
Première brique matérielle de `nexora-vision` : ouvrir la webcam avec `nokhwa`, récupérer des frames RGB et mesurer résolution, FPS et coût de décodage. Pas encore de modèle ONNX : le `HandTracker` réel viendra au spike suivant, avec `ort`.

## Context
- La machine de dev est un **Mac Apple Silicon sous Asahi Linux** (aarch64, Wayland). Ce que montre `v4l2-ctl` :
  - `/dev/video0` = `avd`, le **décodeur vidéo matériel**. Ce n'est **pas** une caméra.
  - `/dev/video1` = **FaceTime HD Camera** (driver `apple-isp`). Seul format : **NV12**, de 320x180 à 1280x720, à 30 fps.
  - L'utilisateur est dans le groupe `video` : pas besoin de root.
- Conséquences :
  1. On ne peut pas supposer « index 0 = webcam ». Il faut détecter automatiquement la première caméra qui s'ouvre réellement.
  2. Il faut gérer le **NV12**, et pas seulement MJPEG/YUYV comme sur la plupart des webcams USB. nokhwa 0.10 sait décoder le NV12 en RGB. Si son backend V4L refuse le fourcc NV12, on bascule sur le crate `v4l` (voir Risques).
- Version disponible : `nokhwa = 0.10.11`. `clang` et `v4l-utils` sont installés.
- Les machines des coéquipiers (Windows, webcams USB) doivent fonctionner aussi. Le backend natif nokhwa (`input-native`) couvre V4L2 sous Linux et Media Foundation sous Windows.
- La PR #1 (`feat/engine-simulation`) a été mergée. On part donc de `main` à jour, qui contient déjà `GestureTracker`, `ScriptedTracker`, `synthetic` et `nexora-engine`.

## Files to modify / create
- `crates/nexora-vision/Cargo.toml`
  - Ajouter une feature `camera = ["dep:nokhwa"]`, **désactivée par défaut**. La CI et les tests restent ainsi sans dépendance système.
  - Ajouter `nokhwa = { version = "0.10", features = ["input-native"], optional = true }`.
- `crates/nexora-vision/src/camera.rs` (nouveau, `#[cfg(feature = "camera")]`) :
  - `RgbFrame { width, height, data: Vec<u8> }`.
  - `CameraInfo { index, name }` et `list_cameras() -> Result<Vec<CameraInfo>, VisionError>`.
  - `Camera::open(index: Option<u32>, width, height, fps)` :
    - Avec `None`, tente chaque caméra listée et garde la première qui ouvre un flux. Le décodeur `avd` est ainsi ignoré.
    - Le format est demandé via `RequestedFormatType::Closest`.
  - `Camera::frame() -> Result<RgbFrame, VisionError>` décode en RGB. `Camera::info()` renvoie le nom, la résolution et le format effectifs.
  - Les erreurs nokhwa sont converties en `VisionError::Camera(String)` (le type existe déjà).
- `crates/nexora-vision/src/lib.rs` : ajouter `#[cfg(feature = "camera")] pub mod camera;`.
- `crates/nexora-vision/examples/camera_probe.rs` (`required-features = ["camera"]` dans le `Cargo.toml`) :
  - Args : `[index] [frames=150]`.
  - Affiche la liste des caméras, celle retenue, puis la résolution et le format effectifs.
  - Capture N frames et affiche les **FPS mesurés** et le temps moyen de capture + décodage par frame.
  - Écrit la dernière frame en **PPM**, sans dépendance `image`, dans `target/camera_probe.ppm`, pour vérifier visuellement l'image.
- `docs/ARCHITECTURE.md`, `README.md` :
  - Mentionner la feature `camera` et la commande de test.
  - Ajouter la note Asahi (caméra en NV12 sur `/dev/video1`, `/dev/video0` = décodeur).

## Steps
1. `git switch main && git pull`, vérifier que le squash de la PR #1 est présent, puis `git switch -c feat/vision-camera-spike` et supprimer la branche locale `feat/engine-simulation`. Copier ce plan dans `.agents/plans/VISION-CAMERA-SPIKE.md`.
2. Ajouter la dépendance et la feature, puis lancer `cargo build -p nexora-vision --features camera` pour valider que nokhwa compile sur aarch64 (bindgen, v4l).
3. Implémenter `camera.rs` : liste, ouverture avec auto-sélection, `frame()` en RGB.
4. Écrire l'exemple `camera_probe`. Le lancer et ajuster selon le comportement réel (NV12, résolution obtenue).
5. Mettre à jour la doc.
6. Lancer `fmt`, `clippy` (avec et sans `--features camera`) et les tests, puis commiter en `feat(vision): ...`.

## Risques / plan B
- **NV12 non accepté par nokhwa sur `apple-isp`** : implémenter `camera.rs` directement avec le crate `v4l` (mmap stream, fourcc `NV12`) et une conversion NV12 → RGB maison (≈ 20 lignes, BT.709). L'API publique `Camera`/`RgbFrame` reste identique.
- **Échec de build de nokhwa sur aarch64** : même plan B.
- La CI ne compile pas la feature `camera`. On pourra ajouter un job `--features camera` plus tard (rôle Infra).

## Conventions
- Commentaires et messages en français, style du crate (`thiserror`, doc `//!`).
- Couche matérielle derrière la feature. Le mock existant (`HandTracker`) reste utilisable sans caméra.
- Pas de nouveau test unitaire dépendant du matériel. La vérification passe par l'exemple.

## Verification
- `cargo test --workspace` (sans feature) reste vert.
- `cargo clippy -p nexora-vision --all-targets --features camera -- -D warnings`.
- `cargo run -p nexora-vision --features camera --example camera_probe` doit :
  - détecter la FaceTime HD sur l'index 1 ;
  - annoncer environ 30 fps en 640x480 ou 1280x720 ;
  - écrire `target/camera_probe.ppm`, qu'on ouvre dans une visionneuse pour vérifier que l'image est correcte (couleurs, orientation).

## Résultat du spike
- `nokhwa` détecte la caméra mais échoue à l'ouverture sur `apple-isp` : `Framerate not whole number: 7680 / 256`. Plan B appliqué : backend `v4l` direct sous Linux (NV12 + YUYV, conversion RGB maison, testée), `nokhwa` conservé pour les autres OS (non testé ici).
- Mesuré sur la FaceTime HD : 640x480 NV12, ~25 fps identiques en debug et en release, donc limité par la caméra et non par le décodage. Image vérifiée visuellement (couleurs et orientation correctes, non miroir).
