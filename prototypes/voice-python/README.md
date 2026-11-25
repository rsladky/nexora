# Prototype Python de la reconnaissance vocale (VocaReco)

Piloter le PC à la voix : ouvrir des applis, aller sur des sites, chercher sur internet, dicter du texte, et les raccourcis classiques (« copier » → **Ctrl+C**).
Fonctionne **hors ligne** (Whisper tourne sur le PC, rien n'est envoyé sur internet).
L'assistant peut **reconnaître ta voix** et ignorer les commandes des autres personnes.

> **Place dans Nexora.** Ce dossier est un **prototype autonome, Windows uniquement**, qui marche déjà de bout en bout (micro → action).
> Il sert à tester en conditions réelles (salle de cours bruyante) et de **référence pour le crate Rust `nexora-voice`**, qui reste la cible (« tout Rust à l'exécution », voir `docs/ARCHITECTURE.md`).
> Il ne dépend d'aucun crate et aucun crate ne dépend de lui : la CI Rust n'est pas concernée.
> Ce qu'il faut reprendre côté Rust est résumé dans [`docs/VOICE_COMMANDS.md`](../../docs/VOICE_COMMANDS.md#prototype-python-et-portage-rust) et à la fin de ce fichier.

## Comment ça marche

```
micro ──► découpage ──► c'est bien ta voix ? ──► Whisper ──► texte ──► commande ──► action
         (micro.py)     (voix.py)                 (transcription.py)   (commandes.py) (actions.py, fonctions.py)
```

1. **micro.py** écoute en continu. Il mesure le bruit de fond au démarrage, puis dès que le volume monte, il enregistre jusqu'à 0,6 s de silence.
2. **voix.py** compare la voix entendue à ta voix enregistrée. Si ce n'est pas toi, la phrase est ignorée (avant même Whisper).
3. **transcription.py** donne ce morceau d'audio à Whisper, qui renvoie le texte (« Ouvre Spotify. »).
4. **commandes.py** nettoie le texte (minuscules, sans accents) et cherche la commande correspondante. Il tolère les petites erreurs (« Coles » → coller).
5. **actions.py** exécute : un raccourci clavier, ou une fonction de **fonctions.py** (ouvrir une appli, un site…). **actions.py est commun avec la partie gestes.**
   **applis.py** sait quelles applis sont installées et quels sites on connaît ; **texte.py** contient les outils de nettoyage du texte.

Protection contre les déclenchements involontaires : une commande simple de plus de 4 mots est ignorée.
« Copier » déclenche Ctrl+C, mais « je vais copier ce texte » ne fait rien.

Ce qui est toléré autour d'une commande :
- les hésitations au début : « Euh, bon, ouvre Spotify » ;
- ce qui est dit APRÈS la commande (toi ou un voisin) : « Enregistre, fais d'accord » → enregistrer, « Va sur YouTube, tu vas voir » → YouTube, « Ouvre Spotify s'il te plaît » → Spotify ;
- mais pas une commande au milieu d'une phrase : « …il est capable de, imprimer, … » ne fait rien (c'est quelqu'un qui parle, pas un ordre).
- Les phrases que Whisper invente sur du bruit (« Merci d'avoir regardé », « Sous-titres réalisés par… ») sont supprimées.

## Dans une salle bruyante (cours, open space)

Tests faits avec 20 commandes (voix de synthèse) noyées dans une conversation :

| Situation | `base` (par défaut) | `small` |
|---|---|---|
| Calme | 16/20 | 19/20 |
| Conversation 3x moins forte que toi | 8/20 | 15/20 |
| Conversation aussi forte que toi | 2/20 | 2/20 |

Ce qui change vraiment les résultats, par ordre d'importance :
1. **Être plus fort que la salle** : micro-casque ou parler près du micro du portable. Le logiciel ne peut pas séparer deux voix qui parlent en même temps au même volume.
2. **Enregistrer ta voix** (`python interface_voix.py`) : les phrases des autres sont jetées avant Whisper, il ne perd plus de temps à transcrire la salle.
3. `--modele small` : beaucoup plus précis dans le bruit, mais ~3 s par commande sur un PC sans carte graphique (plus si le processeur est occupé).

Le micro remesure le bruit de fond en continu (5 dernières secondes) : quand la salle devient bruyante, il faut parler plus fort que le fond pour déclencher, les voix lointaines passent moins.
Le son qui arrive pendant une transcription n'est plus perdu (jusqu'à 3 s de retard, au-delà on oublie le plus ancien).

## Installation

```bash
pip install -r requirements.txt
```

Au premier lancement, deux modèles sont téléchargés automatiquement : Whisper `base` (~140 Mo) et le modèle de reconnaissance de voix (~30 Mo, dans `modeles/`).

## Lancer

**1. Enregistrer ta voix** (une seule fois, conseillé) :

```bash
python interface_voix.py
```

- « Enregistrer ma voix » : lis les 6 phrases affichées, normalement, à ta distance habituelle du micro.
- « Commencer le test » : parle, puis fais parler quelqu'un d'autre. Tu dois voir ✅ pour toi et ❌ pour l'autre.
- « Sévérité » : si tu es parfois refusé, baisse-la ; si l'autre passe, monte-la.
- La case « Ignorer les commandes des autres voix » active/désactive le filtre sans effacer ta voix.

**2. Lancer l'assistant** :

```bash
python main.py --simulation
```

Affiche ce qu'il ferait sans rien exécuter (pour tester). Ensuite, pour de vrai :

```bash
python main.py
```

Reste silencieux 1 seconde au démarrage (calibrage du bruit), puis parle. `Ctrl+C` au clavier dans le terminal pour arrêter.
(La commande vocale « copier » envoie aussi Ctrl+C, mais elle n'arrête pas le programme : il reconnaît que c'est lui qui l'a envoyé.)

## Commandes pratiques (mot-clé + ce que tu veux)

C'est là que la voix est plus utile que les gestes : tout ce qu'on ne peut pas faire en bougeant la main.

| Tu dis | Effet |
|---|---|
| **juste le nom** : « Discord », « YouTube », « la calculatrice » | Ouvre l'application (ou le site) |
| **ouvre** / **lance** + appli : « ouvre Spotify », « lance la calculatrice » | Pareil, et tolère une petite faute de reconnaissance (« Discorde ») |
| **va sur** + site : « va sur YouTube », « va sur ynov point com » | Ouvre le site. Site inconnu → premier résultat de recherche. |
| **cherche** + recherche : « cherche recette de crêpes » | Recherche Google |
| « cherche chats mignons **sur YouTube** » | Recherche sur YouTube (aussi : sur Wikipedia, sur Amazon, sur Google Maps, sur images) |
| **écris** / **tape** + texte : « écris je serai en retard de dix minutes » | Tape le texte là où est le curseur (accents compris) |

**Quelles applis ?** Toutes celles du menu Démarrer de ton PC : la liste est lue automatiquement au lancement (`applis.py`), rien à configurer, et elle s'adapte au PC de chacun.
Les noms courts marchent aussi : « Teams » (Microsoft Teams), « Minecraft » (Minecraft Launcher), « PyCharm » (PyCharm 2023.2.3).

Ordre de recherche d'un nom :
1. nos alias (`APPLICATIONS` dans `applis.py`) : « calculette », « bloc-notes », « mes fichiers »… ;
2. les applis installées ;
3. les sites (`SITES` dans `applis.py`) : si l'appli n'est pas installée, on ouvre son site (« Spotify » → open.spotify.com).

Les petites fautes de Whisper sont tolérées (« Painte » → Paint, « Discorde » → Discord). Pour éviter d'ouvrir des applis au hasard, un nom dit tout seul fait 3 mots maximum et doit ressembler à 90 % au vrai nom (« world » n'ouvre pas Word). Après « ouvre », 85 % suffisent : si un nom seul n'est pas compris, dis « ouvre … ».

## Commandes simples

Astuce : les formes longues (« coller », « effacer ») sont mieux reconnues que les formes courtes (« colle », « efface »).

**Édition**

| Tu dis | Raccourci |
|---|---|
| copier / coller / couper | Ctrl+C / Ctrl+V / Ctrl+X |
| annuler / rétablir (refaire) | Ctrl+Z / Ctrl+Y |
| tout sélectionner | Ctrl+A |
| enregistrer (sauvegarder) | Ctrl+S |
| rechercher (seul) | Ctrl+F |
| nouveau / ouvrir (seul) / imprimer | Ctrl+N / Ctrl+O / Ctrl+P |
| gras / italique / souligner | Ctrl+B / Ctrl+I / Ctrl+U |
| presse-papiers | Win+V (historique des copies) |

**Touches**

| Tu dis | Touche |
|---|---|
| entrée (valider) / échap / espace / tabulation | Entrée / Échap / Espace / Tab |
| effacer (retour arrière) | ⌫ efface le caractère avant le curseur |
| supprimer | Suppr, efface le caractère après le curseur |
| haut / bas / gauche / droite (ou « flèche du bas »…) | Flèches |
| début / fin (de ligne) | Début / Fin |
| monter / descendre | Page précédente / Page suivante |

**Navigateur**

| Tu dis | Raccourci |
|---|---|
| nouvel onglet / fermer l'onglet / rouvrir l'onglet | Ctrl+T / Ctrl+W / Ctrl+Maj+T |
| onglet suivant / onglet précédent | Ctrl+Tab / Ctrl+Maj+Tab |
| retour (page précédente) / page suivante | Alt+← / Alt+→ |
| actualiser (recharger) | F5 |
| zoomer / dézoomer (zoom arrière) | Ctrl+ + / Ctrl+ − |
| plein écran | F11 |

**Présentation** (PowerPoint, Google Slides, PDF)

| Tu dis | Touche |
|---|---|
| lance le diaporama (lance la présentation) | F5 |
| diapo suivante / diapo précédente | → / ← |
| écran noir (re-dire pour revenir) | B |
| quitte le diaporama (fin du diaporama) | Échap |

**Fenêtres et Windows**

| Tu dis | Raccourci |
|---|---|
| changer de fenêtre / fermer la fenêtre | Alt+Tab / Alt+F4 |
| agrandir / réduire | Win+↑ / Win+↓ |
| fenêtre à gauche / fenêtre à droite | Win+← / Win+→ (moitié d'écran) |
| bureau | Win+D |
| menu démarrer | Win |
| explorateur (mes fichiers) | Win+E |
| paramètres (réglages) | Win+I |
| gestionnaire des tâches | Ctrl+Maj+Échap |
| verrouiller | Verrouille la session (⚠️ pour de vrai) |
| capture d'écran | Win+Maj+S |

**Son et musique**

| Tu dis | Touche |
|---|---|
| monte le son / baisse le son / muet | Volume |
| lecture / pause | Lecture-pause |
| musique suivante / musique précédente (ou « morceau ») | Piste suivante / précédente |

**Contrôle de l'assistant**

| Tu dis | Effet |
|---|---|
| arrête l'écoute / reprends l'écoute | Met l'assistant en pause (utile pendant une présentation) |

Ajouter une commande simple = 2 lignes :
- dans `actions.py`, le raccourci : `"imprimer": ["ctrl", "p"],`
- dans `commandes.py`, les mots (sans accents, 4 mots max) : `"imprimer": ["imprimer", "imprime"],`

Ajouter une commande avec mot-clé : une fonction dans `fonctions.py`, son nom dans `FONCTIONS` (`actions.py`) et ses mots-clés dans `MOTS_CLES` (`commandes.py`).

Si Whisper a du mal avec un mot, l'ajouter aussi dans `INDICE_WHISPER` (dans `commandes.py`, en vrai français cette fois).

## Reconnaissance de ta voix : ce qu'il faut savoir

- Le modèle (CAM++) transforme la voix en « empreinte » de 192 nombres qui décrit son timbre, quels que soient les mots. Ta voix enregistrée = la moyenne des empreintes de tes 6 phrases, sauvegardée dans `profil_voix.npz`.
- Testé avec 3 voix de synthèse : la voix enregistrée obtient 0,51 à 0,81, les autres 0,11 à 0,47. La sévérité par défaut (0,50) les sépare. Avec un vrai micro, règle-la avec le test de l'interface.
- Les mots très courts (« copier » seul) donnent des scores moins stables que les phrases longues.
- **Ce n'est pas un système de sécurité** : un enregistrement de ta voix passerait, et une voix très proche aussi. C'est un filtre contre les déclenchements par les gens autour.
- `profil_voix.npz` est une donnée personnelle : il est dans le `.gitignore`, chacun enregistre la sienne sur son PC.

## Lien avec le reste de Nexora

Dans Nexora, les gestes et la voix passent par le même chemin : `nexora-vision` produit un `GestureEvent`, `nexora-voice` produit un `VoiceIntent`, le `Mapper` de `nexora-core` les transforme en `Action` et `nexora-input` les exécute.
Dans ce prototype, ces étapes correspondent à :

| Prototype Python | Nexora (Rust) |
|---|---|
| `micro.py` (découpage par volume) | `nexora-voice` : capture `cpal` + VAD (à venir) |
| `transcription.py` (faster-whisper) | `nexora-voice` : `SpeechRecognizer` avec `whisper-rs` (à venir) |
| `commandes.py` → `trouver_action()` | `nexora-voice::parse_intent()` |
| nom d'action (`"copier"`, `"ouvrir_appli"`…) | `VoiceIntent` (`Copy`, `Type(String)`…) dans `nexora-core` |
| `actions.py` / `fonctions.py` | `Mapper` + `nexora-input` |
| `voix.py` (est-ce bien ta voix ?) | rien pour l'instant : filtre à ajouter avant Whisper |

`actions.executer(nom, argument)` reste utilisable par un autre script Python : il installe une protection pour que la commande « copier » (Ctrl+C) ne ferme pas le programme qui tourne dans un terminal.
`AssistantVocal().demarrer()` lance l'écoute dans un thread, et `AssistantVocal(afficher=ma_fonction)` envoie les messages à une interface plutôt qu'à `print`.

## Pour le portage Rust : ce qu'on a appris en salle

Mesuré en vrai et sur des enregistrements bruités. Ces chiffres et ces pièges valent aussi pour `whisper-rs`.

- **Seuil de parole adaptatif.** On garde les volumes des 5 dernières secondes et on prend le 20ᵉ centile comme bruit de fond (le niveau des pauses entre les mots). Le seuil vaut 3 fois ce bruit, et il est figé pendant une phrase pour ne pas la couper. Un seuil fixe mesuré au démarrage ne tient pas dès que la salle se met à parler.
- **Garder 300 ms avant le début détecté**, sinon la première syllabe est coupée (« opier »). Une phrase se termine après **0,6 s de silence**, dure au plus 5 s, et tout ce qui fait moins de 0,25 s est ignoré.
- **Ne pas vider la file audio pendant la transcription**, sinon une commande dite à ce moment-là est perdue. On jette seulement le son en attente depuis plus de 3 s.
- **`initial_prompt` de Whisper** (`INDICE_WHISPER` dans `commandes.py`) : une phrase en vrai français qui contient les mots des commandes. Ça aide beaucoup dans le bruit (2/20 sans, à −10 dB). Mais Whisper recopie parfois les phrases d'exemple du prompt (« Cherche une recette » ressortait en boucle) : n'y mettre que des commandes courtes.
- **Filtres de confiance** : on rejette le texte si `avg_logprob < −1,2` ou `no_speech_prob > 0,6`. Avec −1,0, des commandes justes comme « Colle » (−1,02) étaient perdues.
- **Hallucinations** : sur du bruit, Whisper invente « Merci d'avoir regardé », « Sous-titres réalisés par… », « Abonnez-vous ». On les supprime avec une expression régulière (`transcription.py`).
- **Anti-déclenchement** : une commande simple fait **4 mots au plus**. « Copier » déclenche, « je vais copier ce texte » non. On enlève les hésitations au début (« euh, bon, … ») et on ignore ce qui est dit après la commande (« Va sur YouTube, tu vas voir »).
- **Tolérance aux fautes** : correspondance exacte d'abord (la phrase la plus longue gagne), puis ressemblance ≥ 0,8 (difflib) pour les mots d'au moins 5 lettres. Un nom d'appli dit seul doit ressembler à 90 % (« world » n'ouvre pas Word), à 85 % après « ouvre ».
- **Modèle** : `base` met ≈ 1 s par commande sur un processeur sans carte graphique, `small` ≈ 3 s mais il est bien meilleur dans le bruit. Quand la salle parle aussi fort que toi, aucun des deux ne suffit : seul un micro proche (casque) règle ça.
- **Reconnaissance du locuteur** : le modèle CAM++ (ONNX, ≈ 30 Mo, via sherpa-onnx) donne une empreinte de 192 nombres. On compare à la moyenne de 6 phrases avec la similarité cosinus, seuil 0,5. Le modèle est en ONNX, donc il peut tourner avec `ort` comme les modèles de mains.

## Réglages (`config.py`)

- `MODELE_WHISPER` : `"base"` (≈ 1 s par commande sur processeur) ou `"small"` (plus précis, ≈ 3 s). Testable sans modifier le fichier : `python main.py --modele small`.
- `SEUIL_VOIX` : sévérité par défaut de la reconnaissance de ta voix (ensuite réglable dans l'interface).
- `SILENCE_FIN` : temps de silence qui termine une commande. Plus court = plus réactif, mais risque de couper.
- `MOTS_MAX` : longueur max d'une commande simple.
- Si l'assistant ne réagit pas quand tu parles, ou réagit au bruit : jouer sur `MULTIPLICATEUR_BRUIT` (plus bas = plus sensible) / `SEUIL_MINIMUM`.
- `DUREE_MESURE_BRUIT` : sur combien de secondes le bruit de fond est remesuré. `RETARD_MAX` : retard maximal accepté avant d'oublier du son.

## Tests

```bash
python test_commandes.py
```

Vérifie que les textes reconnus tombent sur la bonne action (sans micro).
