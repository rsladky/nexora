"""Télécharge hand_landmarker.task (MediaPipe) et convertit ses modèles en ONNX.

Produit `models/palm_detection.onnx` et `models/hand_landmark.onnx` (non commités).
Usage : cd ml && uv run --group convert scripts/fetch_models.py
"""

import subprocess
import sys
import urllib.request
import zipfile
from pathlib import Path

URL = (
    "https://storage.googleapis.com/mediapipe-models/hand_landmarker/"
    "hand_landmarker/float16/latest/hand_landmarker.task"
)
ROOT = Path(__file__).resolve().parents[2]
MODELS = ROOT / "models"
WORK = ROOT / "ml" / "data" / "tflite"

CONVERSIONS = {
    "hand_detector.tflite": "palm_detection.onnx",
    "hand_landmarks_detector.tflite": "hand_landmark.onnx",
}


def main() -> None:
    MODELS.mkdir(exist_ok=True)
    WORK.mkdir(parents=True, exist_ok=True)

    task = WORK / "hand_landmarker.task"
    if not task.exists():
        print(f"téléchargement {URL}")
        urllib.request.urlretrieve(URL, task)
    with zipfile.ZipFile(task) as z:
        z.extractall(WORK)

    for src, dst in CONVERSIONS.items():
        out = MODELS / dst
        print(f"conversion {src} -> {out.name}")
        subprocess.run(
            [sys.executable, "-m", "tf2onnx.convert", "--tflite", str(WORK / src),
             "--output", str(out), "--opset", "17"],
            check=True,
        )

    import onnxruntime as ort

    for dst in CONVERSIONS.values():
        sess = ort.InferenceSession(str(MODELS / dst))
        print(f"\n{dst}")
        for i in sess.get_inputs():
            print("  entrée ", i.name, i.shape, i.type)
        for o in sess.get_outputs():
            print("  sortie ", o.name, o.shape, o.type)


if __name__ == "__main__":
    main()
