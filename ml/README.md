# ml/ — entraînement des gestes

Pipeline prévu (Python, `uv`) :

1. `collect_landmarks.py` : webcam + MediaPipe, enregistre `(21×3 landmarks, label)` dans `ml/data/` (ignoré par git).
2. `train_gesture.py` : petit MLP sur landmarks normalisés (translation/échelle invariantes).
3. `export_onnx.py` : exporte vers `models/gesture_classifier.onnx`, consommé par `nexora-vision`.

Labels : `point`, `pinch_index`, `pinch_middle`, `two_fingers`, `fist`, `open_palm`, `none` (cf. `docs/GESTURES.md`).

```sh
cd ml && uv sync
```
