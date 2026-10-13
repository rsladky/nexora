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
