# Catalogue des gestes (MVP)

| Geste | Pose | Action par défaut |
|---|---|---|
| `point` | index levé seul | le curseur suit le bout de l'index |
| `pinch_index` | pouce + index joints | clic gauche |
| `pinch_middle` | pouce + majeur joints | clic droit |
| `two_fingers` | index + majeur levés | scroll selon le mouvement vertical |
| `fist` | poing fermé | glisser-déposer tant que maintenu |
| `open_palm` | paume ouverte | pause / reprise du suivi |

Les mappings sont modifiables dans la config TOML (`Config::gestures`). Le classifieur actuel est à règles (`nexora-vision::classify`) ; il sera remplacé par un MLP entraîné dans `ml/`.

À valider avec l'équipe : stabilité (debounce), main gauche/droite, éclairage faible.
