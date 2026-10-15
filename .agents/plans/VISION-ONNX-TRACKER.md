# Suivi des mains ONNX (`ort`) : paume + 21 landmarks sur les frames webcam

## Task
Brancher un vrai `HandTracker` sur la capture webcam (PR #2) :
1. Le détecteur de paume MediaPipe (ONNX) trouve la main.
2. Le modèle de landmarks (ONNX) donne les 21 points dans la ROI rotatée.
3. Un filtre One-Euro lisse les points.

On obtient alors `Engine::on_frame` alimenté par la caméra réelle. C'est la brique qui manque au POC du 16/12 (« l'index déplace le curseur, pincement = clic »). Les actions restent journalisées (`LogBackend`).

## Context
- **Déjà en place**
  - `camera::Camera::open(index, w, h, fps)` et `frame() -> RgbFrame` en RGB8, dans `crates/nexora-vision/src/camera.rs` (feature `camera`, backend v4l, ~25 fps en 640x480 sur la FaceTime HD Asahi).
  - Le trait `HandTracker::next() -> Option<Landmarks>`, le type `Landmarks = [[f32;3];21]` (normalisé 0..1, image **non miroir**), `classify()` et `GestureTracker`, dans `crates/nexora-vision/src/lib.rs`.
  - `Engine::on_frame` fait le miroir lui-même pour `MoveCursor` (`crates/nexora-engine/src/lib.rs`). Il ne faut donc pas inverser les landmarks.
  - `VisionError::Model(String)` existe déjà.
- **Pas encore là**
  - Aucun modèle dans `models/`, qui est gitignoré (`*.onnx` aussi).
  - Python système en 3.14. `ml/pyproject.toml` n'épingle rien.
- **Machine** : aarch64 (Asahi). `ort` 2.x fournit des binaires `aarch64-unknown-linux-gnu` via `download-binaries`. Plan B : `load-dynamic` + `ORT_DYLIB_PATH` pointant vers le `libonnxruntime.so` de la wheel `onnxruntime` installée dans `ml/.venv`.
- **CI** : `fmt`, `clippy` et `test` sans feature sur Ubuntu et Windows. La logique pure (ancres, décodage, NMS, ROI, warp, One-Euro) reste **hors feature** pour être testée en CI. Seul l'appel `ort` est derrière la feature `onnx`.

## Pipeline (reprend MediaPipe Hands)
1. **Paume** (seulement si aucune main n'est suivie)
   - Letterbox de la frame en 192×192 RGB float 0..1 (NHWC).
   - Sorties : `regressors [1,2016,18]` et `scores [1,2016,1]`.
   - 2016 ancres SSD : strides 8/16/16/16, soit 24×24×2 + 12×12×6, ancres de taille fixe 1.0.
   - Sigmoïde, seuil 0,5, décodage (box + 7 points-clés, échelle 192), NMS pondérée (IoU 0,3), retrait du letterbox. On garde la meilleure paume.
2. **ROI depuis la paume**
   - Rotation = `π/2 − atan2(−(y2−y0), x2−x0)`, avec kp0 = poignet et kp2 = MCP du majeur, normalisée dans [−π, π].
   - Décalage y de −0,5 et échelle 2,6, puis carré sur le grand côté.
3. **Landmarks**
   - Warp affine (bilinéaire) de la ROI rotatée en 224×224 RGB 0..1 (NHWC).
   - Sorties : 63 valeurs (x, y, z en pixels du crop), score de présence, latéralité.
   - Si présence < 0,5 : main perdue. Sinon transformation inverse vers des coordonnées image normalisées. z est divisé par 224 comme x.
4. **Suivi** : à la frame suivante, la ROI est calculée depuis les landmarks, ce qui saute le détecteur de paume tant que la main reste visible.
   - Boîte des points 0, 1, 2, 3, 5, 6, 9, 10, 13, 14, 17, 18.
   - Rotation poignet (0) → MCP du majeur (9).
   - Échelle 2,0, décalage y de −0,1.
5. **One-Euro** sur chaque coordonnée (min_cutoff 1,0, beta 0,01 par défaut). Réinitialisé quand la main est perdue.

## Files to modify / create
- **`ml/.python-version`** (nouveau) : `3.12`.
- **`ml/pyproject.toml`** : groupe optionnel `convert = ["tensorflow", "tf2onnx", "onnxruntime"]` (et `onnxruntime` utile au plan B `load-dynamic`).
- **`ml/scripts/fetch_models.py`** (nouveau)
  - Télécharge `hand_landmarker.task` (bucket `mediapipe-models`) et le dézippe : `hand_detector.tflite` + `hand_landmarks_detector.tflite`.
  - Convertit chaque fichier avec `python -m tf2onnx.convert --tflite ... --opset 17` vers `models/palm_detection.onnx` et `models/hand_landmark.onnx`. On garde le NHWC.
  - Affiche les noms et formes des entrées et sorties (`onnxruntime`) pour vérifier ce que le code Rust attend.
- **`crates/nexora-vision/src/hand/`** (nouveau module, logique pure non gatée)
  - `anchors.rs` : `generate_anchors() -> Vec<[f32;2]>` (2016 centres).
  - `palm.rs` : `PalmDetection { score, cx, cy, w, h, keypoints: [[f32;2];7] }`, `decode(regressors, scores, anchors, thr)` et `weighted_nms(dets, iou)`.
  - `roi.rs`
    - `Roi { cx, cy, size, rotation }` en pixels image, avec `from_palm`, `from_landmarks`, `to_image(px, py)` et `from_image`.
    - `letterbox(&RgbFrame, 192) -> (Vec<f32>, Letterbox)`.
    - `warp(&RgbFrame, &Roi, 224) -> Vec<f32>`, en bilinéaire et sans dépendance `image`.
  - `filter.rs` : `OneEuro` scalaire + `LandmarkFilter` (63 filtres, `reset()`).
  - `mod.rs` : réexports. Plus `HandResult { landmarks: Landmarks, score, handedness: Handedness, roi: Roi }`.
- **`crates/nexora-vision/src/hand/onnx.rs`** (feature `onnx`)
  - `OnnxHandTracker::new(models_dir: &Path) -> Result<Self, VisionError>` : charge 2 `ort::Session`, en 1 à 2 threads intra-op.
  - `detect(&mut self, &RgbFrame) -> Result<Option<HandResult>, VisionError>`. Gère l'état de suivi (ROI précédente) et le filtre.
  - Le chemin des modèles vient de `NEXORA_MODELS_DIR`, sinon `models/` à la racine du workspace.
- **`crates/nexora-vision/src/hand/live.rs`** (`cfg(all(feature = "camera", feature = "onnx"))`) : `CameraHandTracker { camera, tracker }` implémente `HandTracker`. C'est ce qui se branche directement sur l'`Engine`.
- **`crates/nexora-vision/Cargo.toml`**
  - `ort = { version = "2.0.0-rc.<dernière>", optional = true }`, avec `ndarray` si l'API de la version retenue l'exige.
  - Feature `onnx = ["dep:ort"]`.
  - Exemple `hand_probe` (`required-features = ["camera", "onnx"]`).
- **`crates/nexora-vision/examples/hand_probe.rs`** (nouveau)
  - Ouvre la caméra et fait tourner `OnnxHandTracker` sur N frames.
  - Affiche les FPS, le taux de frames avec une main, le geste `classify()` et le bout de l'index.
  - Dessine les 21 points et la ROI dans la dernière frame avec une main, puis l'écrit dans `target/hand_probe.ppm`.
- **`crates/nexora-vision/src/lib.rs`** : `pub mod hand;`. On met aussi à jour le doc `//!`, qui mentionne nokhwa et `ort` « à venir ».
- **`crates/nexora-engine/Cargo.toml`** + **`examples/live.rs`** (nouveau)
  - Feature `live = ["nexora-vision/camera", "nexora-vision/onnx"]`.
  - L'exemple `live` boucle `CameraHandTracker` → `Engine<LogBackend>::on_frame` et affiche geste et actions (`MoveCursor` / `Click`). C'est la démo du POC sans UI.
- **`docs/ARCHITECTURE.md`**, **`ml/README.md`**, **`README.md`** : feature `onnx`, `fetch_models.py`, commandes `hand_probe` / `live` et plan B `ORT_DYLIB_PATH`.

## Steps
0. Préparer la branche : `git switch main && git pull`, supprimer la branche locale `feat/vision-camera-spike` (déjà mergée), puis `git switch -c feat/vision-onnx-tracker`. Copier ce plan dans `.agents/plans/VISION-ONNX-TRACKER.md`.
1. **Modèles** : `uv python install 3.12`, `.python-version`, groupe `convert`, `fetch_models.py`.
   - Lancer `cd ml && uv run --group convert scripts/fetch_models.py`.
   - Noter les noms et formes réels des I/O dans le plan.
2. **Logique pure + tests unitaires** (sans feature, donc en CI) :
   - ancres : 2016 en tout, premier centre (1/48, 1/48) ;
   - décodage d'une ancre synthétique ;
   - NMS : deux boîtes qui se recouvrent → 1, deux disjointes → 2 ;
   - ROI : aller-retour `to_image`/`from_image` à 1e-4 près, rotation 0 pour une main verticale ;
   - `warp` d'une image unie, plus l'identité avec rotation 0 ;
   - One-Euro : une entrée constante reste constante et le bruit est atténué.
3. **`ort`** : ajouter la dépendance et `onnx.rs`, puis `cargo build -p nexora-vision --features onnx`. Si les binaires aarch64 manquent, passer en `load-dynamic`.
4. **`hand_probe`** en `--release` : vérifier les FPS et l'image PPM (points sur les doigts). Ajuster les seuils et les conventions de sortie si besoin (sigmoïde déjà appliquée ou non, ordre des sorties).
5. **Exemple `live`** dans `nexora-engine` : vérifier que le pointage de l'index produit des `MoveCursor` cohérents (miroir) et qu'un pincement produit `Click`.
6. Mettre à jour la doc. Lancer `cargo fmt`, `clippy` (sans feature, puis avec `--features camera,onnx` sur nexora-vision et `--features live` sur nexora-engine) et `cargo test --workspace`. Commiter en `feat(vision): suivi des mains ONNX (paume + landmarks) derrière la feature onnx` et `feat(engine): exemple live caméra -> moteur`. Puis ouvrir la PR.

## Conventions
- Commentaires et messages en français, `thiserror`, doc `//!` par module, comme dans `camera.rs`.
- Matériel et runtime ONNX derrière des features désactivées par défaut. La CI reste sans dépendance native.
- Pas de nouvelle dépendance pour le traitement d'image : letterbox et warp sont faits à la main, comme la conversion NV12.
- Python : `uv`, scripts dans `ml/scripts/`, modèles non commités dans `models/`.

## Risques / plan B
- **`tf2onnx` + TensorFlow en Python 3.12 aarch64** : si l'install ou la conversion échoue, essayer `tflite2onnx` (pur Python). Sinon prendre les ONNX déjà convertis de PINTO_model_zoo (033_Hand_Detection_and_Tracking) et adapter le décodage à leurs sorties.
- **`ort` sans binaire aarch64** : passer à la feature `load-dynamic` + `ORT_DYLIB_PATH`, documentée dans le README.
- **Performances** : si on est sous 20 fps, réduire à 1 thread de capture et 1 d'inférence, utiliser le modèle landmarks « lite » s'il est disponible, et vérifier que la paume n'est relancée qu'en cas de perte.
- **Conventions des sorties** (ordre des tenseurs, sigmoïde, z) : elles sont validées à l'étape 4 avec le dump PPM. On peut comparer avec MediaPipe Python sur une même image si besoin (script de référence optionnel).

## Verification
- `cargo test --workspace` (sans feature) reste vert et inclut les nouveaux tests ancres, NMS, ROI, warp et One-Euro.
- `cargo clippy -p nexora-vision --all-targets --features camera,onnx -- -D warnings` et `cargo clippy -p nexora-engine --all-targets --features live -- -D warnings`.
- `cargo run --release -p nexora-vision --features camera,onnx --example hand_probe -- 1 300` doit :
  - annoncer environ 20–25 fps, la caméra étant le plafond ;
  - détecter une main dans plus de 90 % des frames quand elle est visible ;
  - écrire `target/hand_probe.ppm`, où les 21 points se superposent aux doigts.
- `cargo run --release -p nexora-engine --features live --example live` :
  - index levé → flux de `MoveCursor` dont x augmente quand la main va vers la droite de l'utilisateur ;
  - pincement → `Click`.
