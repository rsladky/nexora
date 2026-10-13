# Contribuer

## Git
- `main` protégée ; branches `feat/…`, `fix/…`, `docs/…` ; PR relue avant merge.
- Commits conventionnels : `feat(vision): …`, `fix(core): …`.

## Rust
- `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` avant chaque PR.
- Erreurs : `thiserror` en bibliothèque, `anyhow` dans l'app. Logs : `tracing`.
- Toute couche matérielle derrière un trait avec un mock.

## JavaScript / TypeScript
- **bun uniquement** (`bun install`, `bun run`, `bunx`), TypeScript strict.

## Scaffolder l'app Tauri (une fois)
```sh
bunx create-tauri-app app --template react-ts --manager bun
```
Puis ajouter `app/src-tauri` aux `members` du `Cargo.toml` racine et dépendre des crates `nexora-*`.

## Données
Modèles (`models/`) et datasets (`ml/data/`) ne sont jamais commités : utiliser `scripts/download-models.sh`.
