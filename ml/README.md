# ml/ — entraînement des gestes

Pipeline prévu (Python, `uv`) :

1. `collect_landmarks.py` : webcam + MediaPipe, enregistre `(21×3 landmarks, label)` dans `ml/data/` (ignoré par git).
2. `train_gesture.py` : petit MLP sur landmarks normalisés (translation/échelle invariantes).
3. `export_onnx.py` : exporte vers `models/gesture_classifier.onnx`, consommé par `nexora-vision`.

Labels : `point`, `pinch_index`, `pinch_middle`, `two_fingers`, `fist`, `open_palm`, `none` (cf. `docs/GESTURES.md`).

```sh
cd ml && uv sync
```

## Modèles de suivi des mains

`scripts/fetch_models.py` télécharge `hand_landmarker.task` (MediaPipe) et le convertit en
`models/palm_detection.onnx` + `models/hand_landmark.onnx` (non commités), consommés par `nexora-vision` (feature `onnx`) :

```sh
cd ml && uv run --group convert scripts/fetch_models.py   # Python 3.12 (ml/.python-version)
```
