# TODO

## Plus tard

- [ ] **Viewer de détections dans l'app (POC)** : flux webcam dans Tauri + React, avec par-dessus :
  - le rectangle de paume rotaté et ses 7 points-clés ;
  - les 21 landmarks et le squelette ;
  - le geste courant et les actions journalisées.

  La source est un sidecar MediaPipe Python, qui fait aussi tourner le détecteur de paume ONNX (`palm.py`). Ce code sert de référence pour le futur portage `ort`.
  Plan détaillé : [`plans/DETECTION-VIEWER-POC.md`](plans/DETECTION-VIEWER-POC.md).
