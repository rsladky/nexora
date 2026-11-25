"""Réglages de la reconnaissance vocale (modifiables sans toucher au reste du code)."""

# Modèle Whisper : "tiny", "base", "small", "medium"...
# Plus il est gros, plus il est précis mais lent (ici sur processeur, sans carte graphique).
# Mesuré sur un PC portable sans carte graphique : "base" ≈ 1 s par commande, "small" ≈ 3 s.
MODELE_WHISPER = "base"
LANGUE = "fr"

# Micro
FREQUENCE = 16000          # Whisper travaille en 16 kHz
DUREE_BLOC = 0.03          # on analyse le son par morceaux de 30 ms

# Détection de la parole (quand commence / finit une phrase)
DUREE_CALIBRAGE = 1.0      # secondes de silence mesurées au démarrage pour connaître le bruit de fond
DUREE_MESURE_BRUIT = 5.0   # ensuite, le bruit de fond est remesuré en continu sur les 5 dernières secondes
RETARD_MAX = 3.0           # au-delà de 3 s de son en attente de traitement, on oublie le plus ancien
MULTIPLICATEUR_BRUIT = 3.0 # on considère qu'on parle quand le volume dépasse 3x le bruit de fond
SEUIL_MINIMUM = 0.01       # seuil de volume plancher (si la pièce est très calme)
SILENCE_FIN = 0.6          # secondes de silence qui marquent la fin d'une commande
DUREE_MIN = 0.25           # en dessous, c'est un bruit (clic, toux...), on ignore
DUREE_MAX = 5.0            # une commande ne dure pas plus longtemps
PRE_ENREGISTREMENT = 0.3   # on garde 300 ms avant le début détecté pour ne pas couper le 1er son

# Reconnaissance de TA voix (voir voix.py et interface_voix.py)
# Score de ressemblance minimal pour accepter une commande. Plus haut = plus strict
# (les autres passent moins, mais toi tu risques d'être refusé plus souvent).
# Valeur par défaut, réglable ensuite dans l'interface.
SEUIL_VOIX = 0.5

# Anti-déclenchement : une commande est une phrase COURTE.
# Si tu parles normalement ("je vais copier ce texte"), on n'exécute rien.
MOTS_MAX = 4
