# Nexora — création du repo GitHub + premier commit

## Task
Créer le repo GitHub `rsladky/nexora` (privé) et pousser un premier commit avec le bootstrap existant. La préparation du workflow git (protection de branche, templates, etc.) viendra ensuite.

## Context
- Repo local `/home/kinkin/Etude/Nexora` déjà initialisé (`main`), rien de commité : workspace Cargo (4 crates, 12 tests verts), docs, scripts, CI, `ml/`, `.agents/plans/`.
- `gh` connecté en `rsladky` (scopes `repo`), protocole ssh. Identité git : `RSLADKY <robinsladky@gmail.com>`.
- `.gitignore` exclut `target/`, `node_modules/`, `models/*` (sauf `.gitkeep`), `ml/.venv`, `ml/data`, `*.onnx`, `*.bin`.

## Steps
1. Vérifier ce qui sera commité : `git add -A && git status --short` (s'assurer qu'aucun `target/` ni binaire n'est inclus ; `Cargo.lock` est commité, c'est une app).
2. Commit : `chore: bootstrap du repo Nexora (workspace Rust, docs, CI, ml)` avec la ligne `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
3. `gh repo create rsladky/nexora --private --source . --remote origin --description "Nexora — contrôle de l'ordinateur par gestes et voix (IA). Projet YDAYS 2026-2027." --push`.
4. Mettre à jour `.agents/plans/NEXORA-DESKTOP-BOOTSTRAP.md` ? Non — on ajoute ce plan en `.agents/plans/GITHUB-REPO-SETUP.md` avant le commit.

## Verification
- `git log --oneline` montre le commit ; `git remote -v` pointe vers `git@github.com:rsladky/nexora.git`.
- `gh repo view rsladky/nexora --json visibility,defaultBranchRef` → `PRIVATE`, `main`.
- L'onglet Actions lance la CI sur le push (vérifier avec `gh run list --limit 1`).
