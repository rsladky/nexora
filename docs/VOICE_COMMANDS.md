# Commandes vocales (MVP, français)

Préfixe « Nexora » optionnel. Accents et ponctuation ignorés.

| Phrase | Intention | Action par défaut |
|---|---|---|
| copie | `copy` | Ctrl+C |
| colle | `paste` | Ctrl+V |
| coupe | `cut` | Ctrl+X |
| annule | `undo` | Ctrl+Z |
| sélectionne tout | `select_all` | Ctrl+A |
| nouvel onglet | `new_tab` | Ctrl+T |
| ferme / ferme l'onglet | `close_tab` | Ctrl+W |
| défile vers le haut / bas | `scroll_up` / `scroll_down` | scroll |
| clic droit | `right_click` | clic droit |
| capture d'écran | `screenshot` | Impr. écran |
| ouvre le navigateur | `open_browser` | lance le navigateur |
| volume plus / moins | `volume_up` / `volume_down` | touches volume |
| tape … / écris … | `type` | dictée du texte |

Parser : `nexora-voice::parse_intent`. À prévoir : variantes de formulation, tolérance aux erreurs de transcription (distance d'édition), mode push-to-talk vs écoute continue.

## Prototype Python et portage Rust

Un prototype complet tourne déjà sous Windows dans [`prototypes/voice-python/`](../prototypes/voice-python/README.md). Il va de l'écoute continue jusqu'à l'action, et il a été testé en salle de cours.
Il couvre plus que le MVP ci-dessus :
- environ 80 commandes simples (édition, touches, navigateur, diaporama, fenêtres, son) ;
- des commandes à argument : « ouvre / lance … » (applis installées détectées automatiquement), « va sur … » (sites), « cherche … [sur YouTube] » et « écris / tape … » ;
- le nom d'une appli dit seul (« Discord », « la calculatrice ») ;
- un filtre qui ignore les autres voix que celle de l'utilisateur.

Il sert de **référence** pour `nexora-voice`. Le runtime cible reste en Rust (voir `ARCHITECTURE.md`).

Points à reprendre dans `nexora-voice`. Le détail et les chiffres sont dans le README du prototype, section « Pour le portage Rust ».

| Sujet | Réglage validé dans le prototype |
|---|---|
| VAD | seuil = 3 × le 20ᵉ centile du volume des 5 dernières secondes (figé pendant une phrase) ; 300 ms gardées avant le début ; fin après 0,6 s de silence ; phrase de 0,25 s à 5 s |
| File audio | ne pas la vider pendant la transcription ; jeter seulement ce qui attend depuis plus de 3 s |
| Whisper | `base` int8 (≈ 1 s sur CPU) ; `initial_prompt` avec le vocabulaire des commandes, sans phrases d'exemple longues ; rejet si `avg_logprob < -1.2` ou `no_speech_prob > 0.6` ; filtre des hallucinations (« Merci d'avoir regardé »…) |
| Parser | 4 mots max pour une commande simple ; hésitations retirées au début, suite de phrase ignorée ; correspondance exacte (la plus longue gagne) puis distance d'édition (ressemblance ≥ 0,8, mots de 5 lettres et plus) |
| Locuteur | empreinte CAM++ (ONNX, compatible `ort`), similarité cosinus avec la moyenne de 6 phrases, seuil 0,5 ; profil = donnée personnelle, jamais commitée |

Le prototype utilise des noms d'action en français (`"copier"`, `"ouvrir_appli"`…). Le passage aux `VoiceIntent` demandera de nouvelles variantes dans `nexora-core`, par exemple `OpenApp(String)`, `OpenSite(String)`, `Search(String)` et des raccourcis génériques. C'est à proposer dans une PR dédiée à `nexora-core`, comme le demande `CONTRIBUTING.md`.
