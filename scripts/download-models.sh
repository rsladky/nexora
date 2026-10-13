#!/usr/bin/env bash
# Télécharge les modèles dans models/. Ne commit jamais les fichiers.
set -euo pipefail
cd "$(dirname "$0")/../models"

# Whisper (ggml) multilingue, taille "base" : bon compromis latence/précision en FR.
WHISPER_URL="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
[ -f ggml-base.bin ] || curl -L --fail -o ggml-base.bin "$WHISPER_URL"

# Modèles mains MediaPipe en ONNX : TODO — choisir la source (PINTO model zoo) et
# fixer le hash ici une fois les modèles validés par l'équipe Data IA.
echo "TODO: palm_detection.onnx et hand_landmark.onnx (voir docs/ARCHITECTURE.md)"
