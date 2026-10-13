# Nexora — Gestes · Voix · IA

Application desktop (Windows + Linux) pour contrôler son ordinateur avec les gestes de la main (webcam) et la voix. Projet YDAYS 2026-2027 (semaine C), Ynov Campus.

**Objectif MVP** : ≥ 5 gestes et ≥ 10 commandes vocales fiables, mappings personnalisables, tests utilisateurs.

## Stack

| Couche | Techno |
|---|---|
| App desktop | Tauri v2 (Rust) + React/TypeScript (Vite, bun) |
| Gestes | caméra `nokhwa` → modèles mains ONNX via `ort` → classifieur |
| Voix | `cpal` → `whisper-rs` (FR) → parser d'intentions |
| Actions système | `enigo` (Windows/X11), `uinput` (Wayland) |
| Entraînement | Python (`uv`) dans `ml/`, export ONNX |

## Structure

```
crates/nexora-core     types, Mapper événements→actions, config TOML
crates/nexora-vision   landmarks + classifieur de gestes
crates/nexora-voice    reconnaissance vocale + parser FR
crates/nexora-input    exécution des actions (souris/clavier)
app/                   Tauri + React (à scaffolder, voir docs/CONTRIBUTING.md)
ml/                    collecte de données + entraînement (Python)
docs/                  architecture, catalogues, roadmap
```

## Démarrage

```sh
# Linux (Arch) : dépendances système
sudo pacman -S --needed cmake webkit2gtk-4.1 base-devel

cargo test --workspace            # tests des crates
./scripts/download-models.sh      # modèles ONNX / Whisper dans models/
cd app && bun install && bun run tauri dev
```

Wayland : voir `scripts/setup-linux.sh` pour l'accès à `/dev/uinput`.

## Docs

- [Architecture](docs/ARCHITECTURE.md) · [Gestes](docs/GESTURES.md) · [Commandes vocales](docs/VOICE_COMMANDS.md)
- [Roadmap](docs/ROADMAP.md) · [Contribuer](docs/CONTRIBUTING.md)
