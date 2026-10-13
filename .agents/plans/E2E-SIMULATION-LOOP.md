# Boucle de bout en bout simulée (premier spike d'intégration)

## Task
Relier les briques existantes (vision → Mapper → input, voix → Mapper → input) dans une boucle exécutable **sans matériel**, pour voir le pipeline complet fonctionner et poser la base du futur `engine_start` Tauri.

## Context
- Aujourd'hui chaque crate est testé isolément : `classify()` (nexora-vision), `parse_intent()` (nexora-voice), `Mapper` (nexora-core), `LogBackend` (nexora-input). Rien ne les relie.
- Deux trous concrets apparaissent quand on les relie :
  1. `classify()` renvoie un geste **par frame**, alors que `Mapper` attend des `Event::GestureStart/End` → il manque une machine à états avec debounce (déjà notée « à valider » dans `docs/GESTURES.md`).
  2. Le geste `point` ne produit aucune action : le `MoveCursor` doit venir de la position du bout de l'index, pas de la config.
- Machine de dev : Wayland, webcam présente, `/dev/uinput` en root uniquement → on reste sur `LogBackend` pour ce spike.

## Files to modify / create
- `crates/nexora-vision/src/lib.rs` : ajout de `GestureTracker` (debounce) + module public `synthetic` (main synthétique, déplacé depuis les tests) + `ScriptedTracker` (impl `HandTracker` qui rejoue une séquence).
- `crates/nexora-engine/` (nouveau crate, pris automatiquement par `members = ["crates/*"]`) :
  - `Cargo.toml` : deps `nexora-core`, `nexora-vision`, `nexora-voice`, `nexora-input` (paths).
  - `src/lib.rs` : `Engine<B: InputBackend>`.
  - `examples/simulate.rs` : démo interactive.
  - `tests/pipeline.rs` : test d'intégration.
- `docs/ARCHITECTURE.md` : mentionner `nexora-engine`.
- `.agents/plans/E2E-SIMULATION-LOOP.md` : copie de ce plan (convention du repo).

## Steps
1. **`synthetic` (nexora-vision)** : rendre public le helper `hand([bool; 4]) -> Landmarks` des tests + `pinch(...)`, réutilisé par les tests existants, le tracker scripté et l'exemple.
2. **`GestureTracker` (nexora-vision)** : `update(Option<Gesture>) -> Vec<Event>` ; un geste n'est validé qu'après `N` frames consécutives identiques (défaut 3) ; émet `GestureEnd(ancien)` puis `GestureStart(nouveau)` au changement. Tests unitaires : bruit d'une frame ignoré, transition Fist→None donne `GestureEnd(Fist)`.
3. **`ScriptedTracker`** : `HandTracker` qui renvoie une `Vec<Option<Landmarks>>` puis `None`. Sert de mock caméra.
4. **`Engine` (nexora-engine)** :
   - `new(config: Config, backend: B)` → contient `Mapper`, `GestureTracker`, backend.
   - `on_frame(Option<&Landmarks>)` : `classify` → tracker → `mapper.handle` → `backend.execute` ; si le geste stable est `Point` (et pas en pause), émet aussi `MoveCursor { x: 1.0 - tip.x, y: tip.y }` (miroir webcam).
   - `on_transcript(&str)` : `parse_intent` → `Event::Voice` → mapper → backend.
   - Erreurs `InputError` remontées en `Result`.
5. **`examples/simulate.rs`** : rejoue un scénario de gestes (point qui se déplace → pincement → poing maintenu → relâché → paume = pause), affiche chaque action, puis lit stdin ligne par ligne comme transcriptions (« copie », « tape bonjour »…) et affiche l'action ou « non reconnu ». Lancement : `cargo run -p nexora-engine --example simulate`.
6. **`tests/pipeline.rs`** : scénario fixe vérifiant la liste exacte d'actions dans `LogBackend.executed` (clic, ButtonDown/Up, pause qui bloque « copie », reprise).
7. Mettre à jour `docs/ARCHITECTURE.md`.

## Conventions
- Commentaires et messages en français, style des crates existants (doc `//!` en tête, `thiserror`, tests `#[cfg(test)]`).
- Pas de nouvelle dépendance externe (sortie via `println!` dans l'exemple).
- Commit conventionnel sur une branche dédiée (`feat/engine-simulation`), conformément à `docs/CONTRIBUTING.md`.

## Verification
- `cargo test --workspace` : tests existants plus les nouveaux (tracker, pipeline), tous verts.
- `cargo clippy --workspace --all-targets -- -D warnings` et `cargo fmt --check` (comme la CI).
- `cargo run -p nexora-engine --example simulate`, puis taper `copie`, `nexora sélectionne tout`, `tape bonjour`, `il fait beau` → actions affichées et cas non reconnu.

## Suite possible (spikes suivants)
Remplacer un mock à la fois : webcam `nokhwa` → `HandTracker` réel ; micro `cpal` + `whisper-rs` → `SpeechRecognizer` ; backend `uinput` à la place de `LogBackend`.
