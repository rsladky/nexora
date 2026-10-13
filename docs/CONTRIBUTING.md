# Contribuer

Workflow : **GitHub Flow**. `main` est toujours fonctionnelle ; tout passe par une branche courte et une pull request.

## Branches

- `main` est protégée : pas de push direct.
- Une branche par tâche, nommée `type/zone-description` :
  `feat/vision-onnx-tracker`, `fix/core-pause-toggle`, `docs/roadmap-update`, `ci/windows-cache`.
  Types : `feat`, `fix`, `docs`, `chore`, `ci`, `refactor`, `test`.
- Durée de vie : 1 à 3 jours. Pas de branche par personne.
- Toujours partir de `main` à jour :

```sh
git switch main && git pull
git switch -c feat/voice-whisper
```

## Commits

Commits conventionnels, la zone correspond à un crate ou un dossier :

```
feat(vision): ajoute le filtre One-Euro
fix(core): corrige la reprise après pause
docs: met à jour la roadmap
```

Petits commits fréquents ; pousse ta branche quand tu veux. Avant d'ouvrir la PR :

```sh
git pull --rebase origin main
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

## Pull requests

- Une PR = une tâche, petite (idéalement < 400 lignes). Remplis le template (quoi, pourquoi, comment tester) et lie l'issue (`Closes #12`).
- **1 relecture** approuvée et **CI verte** (Ubuntu + Windows) pour merger.
- Les commentaires de relecture doivent être résolus avant le merge.
- Merge en **squash** uniquement ; la branche est supprimée automatiquement.
- Modifier `nexora-core` (types partagés) : préviens l'équipe avant, via une petite PR dédiée, car elle impacte tous les crates.

## Zones et responsabilités

| Rôle | Zone principale |
|---|---|
| Data IA | `crates/nexora-vision`, `crates/nexora-voice`, `ml/` |
| Dev | `app/`, `crates/nexora-input`, `crates/nexora-core` |
| Infra | `.github/`, `scripts/`, packaging |
| Marketing | `docs/` (hors technique) |

`.github/CODEOWNERS` propose automatiquement les relecteurs selon les dossiers modifiés.

## Issues, labels et suivi

- Les tâches sont des issues (templates *Fonctionnalité* et *Bug*), rattachées à une séance YDAYS.
- Labels de zone : `vision`, `voice`, `app`, `input`, `core`, `ml`, `infra`, `docs`, `marketing`. Labels de type : `feature`, `bug`. Priorité : `P1` (bloquant), `P2`, `P3`.
- Suivi : board GitHub Projects (À faire / En cours / En revue / Fait).

## Versions

Tags sur les jalons : `v0.1.0` (oral de validation 16/12), `v0.2.0` (intégration, mars), `v1.0.0` (oral final). Les artefacts (.msi, AppImage) sont publiés en releases GitHub.

## Rust

- Erreurs : `thiserror` en bibliothèque, `anyhow` dans l'app. Logs : `tracing`.
- Toute couche matérielle (caméra, micro, système) est derrière un trait avec un mock.

## JavaScript / TypeScript

**bun uniquement** (`bun install`, `bun run`, `bunx`), TypeScript strict.

## Scaffolder l'app Tauri (une fois)

```sh
bunx create-tauri-app app --template react-ts --manager bun
```

Puis ajouter `app/src-tauri` aux `members` du `Cargo.toml` racine et dépendre des crates `nexora-*`.

## Données

Modèles (`models/`) et datasets (`ml/data/`) ne sont jamais commités : utiliser `scripts/download-models.sh`.
