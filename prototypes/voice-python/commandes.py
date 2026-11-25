"""
Traduction du texte reconnu en action.

Whisper renvoie du texte ("Copier.", "copie !", "Colle"...).
On le nettoie puis on cherche quelle commande correspond.
"""

import difflib
import re

import applis
import config
from texte import nettoyer, sans_articles

# Ce qu'on peut dire -> nom de l'action (voir actions.py)
# Les phrases sont écrites sans accents ni majuscules (on nettoie le texte avant de comparer).
PHRASES = {
    # --- Édition ---
    "copier":            ["copier", "copie", "copy"],
    "coller":            ["coller", "colle", "paste"],
    "couper":            ["couper", "coupe"],
    "annuler":           ["annuler", "annule"],
    "retablir":          ["retablir", "retablis", "refaire"],
    "tout_selectionner": ["tout selectionner", "selectionne tout", "selectionner tout"],
    "enregistrer":       ["enregistrer", "enregistre", "sauvegarder", "sauvegarde"],
    "rechercher":        ["rechercher", "recherche", "chercher"],
    "nouveau":           ["nouveau", "nouveau fichier", "nouveau document"],
    "ouvrir":            ["ouvrir", "ouvre", "ouvrir un fichier"],
    "imprimer":          ["imprimer", "imprime"],
    "gras":              ["gras", "en gras", "mettre en gras"],
    "italique":          ["italique", "en italique"],
    "souligner":         ["souligner", "souligne"],
    "presse_papier":     ["presse papier", "presse papiers", "historique du presse papier"],

    # --- Touches ---
    "entree":            ["entree", "valider", "valide"],
    "echap":             ["echap", "echappe", "escape"],
    "espace":            ["espace"],
    "tabulation":        ["tabulation", "tab"],
    "supprimer":         ["supprimer", "supprime"],
    "effacer":           ["effacer", "efface", "retour arriere"],
    "haut":              ["haut", "fleche du haut", "fleche haut"],
    "bas":               ["bas", "fleche du bas", "fleche bas"],
    "gauche":            ["gauche", "fleche gauche", "fleche de gauche"],
    "droite":            ["droite", "fleche droite", "fleche de droite"],
    "debut":             ["debut", "debut de ligne"],
    "fin":               ["fin", "fin de ligne"],
    "monter":            ["monter", "monte", "remonter", "remonte"],
    "descendre":         ["descendre", "descends", "descend"],

    # --- Navigateur ---
    "nouvel_onglet":     ["nouvel onglet", "nouvelle page"],
    "fermer_onglet":     ["fermer l'onglet", "ferme l'onglet", "fermer onglet"],
    "rouvrir_onglet":    ["rouvrir l'onglet", "rouvre l'onglet", "rouvrir onglet"],
    "onglet_suivant":    ["onglet suivant"],
    "onglet_precedent":  ["onglet precedent"],
    "page_precedente":   ["page precedente", "precedent", "retour"],
    "page_suivante":     ["page suivante"],
    "actualiser":        ["actualiser", "actualise", "rafraichir", "recharger", "recharge"],
    "zoomer":            ["zoomer", "zoom", "zoom avant"],
    "dezoomer":          ["dezoomer", "dezoom", "zoom arriere"],
    "plein_ecran":       ["plein ecran"],

    # --- Fenêtres et Windows ---
    "fermer_fenetre":    ["fermer la fenetre", "ferme la fenetre", "fermer fenetre"],
    "changer_fenetre":   ["changer de fenetre", "change de fenetre", "fenetre suivante"],
    "agrandir":          ["agrandir", "agrandis", "agrandir la fenetre"],
    "reduire":           ["reduire", "reduis", "reduire la fenetre"],
    "fenetre_gauche":    ["fenetre a gauche", "fenetre gauche"],
    "fenetre_droite":    ["fenetre a droite", "fenetre droite"],
    "bureau":            ["bureau", "afficher le bureau"],
    "menu_demarrer":     ["menu demarrer", "demarrer"],
    "explorateur":       ["explorateur", "explorateur de fichiers", "mes fichiers"],
    "parametres":        ["parametres", "reglages"],
    "gestionnaire_taches": ["gestionnaire des taches", "gestionnaire de taches"],
    "verrouiller":       ["verrouiller", "verrouille", "verrouiller l'ecran"],
    "capture_ecran":     ["capture d'ecran", "capture ecran", "capture"],

    # --- Son et musique ---
    "volume_plus":       ["monte le son", "plus fort", "volume plus"],
    "volume_moins":      ["baisse le son", "moins fort", "volume moins"],
    "muet":              ["muet", "coupe le son", "couper le son"],
    "lecture_pause":     ["lecture", "pause", "play"],
    "morceau_suivant":   ["morceau suivant", "chanson suivante", "musique suivante"],
    "morceau_precedent": ["morceau precedent", "chanson precedente", "musique precedente"],

    # --- Présentation ---
    "lancer_diaporama":  ["lance le diaporama", "lancer le diaporama", "lance la presentation",
                          "lancer la presentation", "diaporama"],
    "diapo_suivante":    ["diapo suivante", "diapositive suivante", "slide suivante"],
    "diapo_precedente":  ["diapo precedente", "diapositive precedente", "slide precedente"],
    "quitter_diaporama": ["quitte le diaporama", "quitter le diaporama", "fin du diaporama",
                          "arrete le diaporama"],
    "ecran_noir":        ["ecran noir"],
}

# Commandes suivies d'un ARGUMENT : un mot-clé au début de la phrase, puis ce qu'on veut.
#   "ouvre Spotify"            -> ouvrir_appli("Spotify")
#   "va sur YouTube"           -> aller_sur("YouTube")
#   "cherche recette de crêpes"-> chercher("recette de crêpes")
#   "écris bonjour à tous"     -> ecrire("bonjour à tous")
MOTS_CLES = {
    "ecrire":       ["ecris", "ecrit", "ecrire", "tape", "taper"],
    "ouvrir_appli": ["ouvre", "ouvrir", "lance", "lancer", "demarre"],
    "aller_sur":    ["va sur", "vas sur", "aller sur", "allez sur"],
    "chercher":     ["cherche", "recherche", "chercher", "rechercher", "google"],
}
MOTS_MAX_ARGUMENT = 25   # ces phrases-là peuvent être longues (dictée, recherche)
MOTS_MAX_NOM_SEUL = 3    # "Spotify", "Visual Studio Code" : un nom d'appli dit tout seul l'ouvre
RESSEMBLANCE_NOM_SEUL = 0.9   # tolérance aux fautes pour un nom dit seul (0.85 après "ouvre")

# Commandes spéciales gérées par l'assistant lui-même (pas de raccourci clavier)
PAUSE_ECOUTE = "pause_ecoute"
REPRISE_ECOUTE = "reprise_ecoute"
PHRASES[PAUSE_ECOUTE] = ["arrete d'ecouter", "arrete l'ecoute", "stop ecoute", "mode veille"]
PHRASES[REPRISE_ECOUTE] = ["reprends l'ecoute", "reprend l'ecoute", "reveille toi", "ecoute moi"]

SIMILARITE_MIN = 0.8   # pour accepter un mot mal reconnu ("copié" -> "copie" ok, "copain" non)
LONGUEUR_MIN_APPROX = 5  # les mots courts ("fin", "bas", "gras") doivent être reconnus exactement,
                         # sinon "fini" déclencherait "fin"


# phrase -> action, pour retrouver une commande simple à partir de sa phrase exacte
_ACTION_DE_LA_PHRASE = {phrase: action for action, phrases in PHRASES.items() for phrase in phrases}


def _commande_a_argument(texte_original):
    """Si la phrase commence par un mot-clé (ouvre, cherche...), renvoie (action, argument)."""
    texte = nettoyer(texte_original)
    mots = texte.split()
    if len(mots) > MOTS_MAX_ARGUMENT:
        return None, None
    for action, mots_cles in MOTS_CLES.items():
        for mot_cle in mots_cles:
            n = len(mot_cle.split())
            if mots[:n] != mot_cle.split() or len(mots) == n:
                continue
            reste = " ".join(mots[n:])
            # "ouvre l'explorateur", "lance le diaporama", "ouvre un nouvel onglet" :
            # ce sont déjà des commandes simples, on les garde telles quelles
            # (sauf la dictée : "écris copier" doit écrire le mot "copier").
            if action != "ecrire":
                for candidat in (texte, reste, sans_articles(reste)):
                    if candidat in _ACTION_DE_LA_PHRASE:
                        return _ACTION_DE_LA_PHRASE[candidat], None
            # L'argument est pris dans le texte ORIGINAL (accents, majuscules) :
            # "Écris, Bonjour à tous." -> "Bonjour à tous."
            argument = " ".join(texte_original.split()[n:])
            return action, argument
    return None, None


# Mots d'hésitation qu'on enlève au début : "Euh, ouvre Spotify" -> "ouvre Spotify"
# (pas "allez" : "allez sur YouTube" est une commande)
HESITATIONS = {"euh", "heu", "hum", "bon", "ben", "bah", "ok", "okay", "alors", "donc",
               "et", "voila", "bref", "hein", "oui", "ouais"}
# Fin de phrase de politesse qu'on enlève des arguments : "ouvre Spotify s'il te plaît"
POLITESSE = re.compile(r"[\s,]*(s'?il (te|vous) pla[iî]t|stp|svp|merci)[\s.!?]*$", re.IGNORECASE)
FIN_DE_PROPOSITION = re.compile(r"[,;:.!?…]")


def _sans_hesitations(texte_original):
    """'Euh, bon, ouvre Spotify.' -> 'ouvre Spotify.' (le texte original, accents gardés)."""
    mots = texte_original.split()
    while mots and all(m in HESITATIONS for m in nettoyer(mots[0]).replace("'", " ").split()):
        mots.pop(0)
    return " ".join(mots)


def _premiere_proposition(texte_original):
    """'Enregistre, fais d'accord.' -> 'Enregistre' : le morceau avant la première ponctuation."""
    return FIN_DE_PROPOSITION.split(texte_original, maxsplit=1)[0].strip()


def _nettoyer_argument(action, argument):
    """Enlève ce que les voisins (ou toi) ont dit après le nom :
    'YouTube, tu vas voir.' -> 'YouTube' ; 'Spotify s'il te plaît' -> 'Spotify'."""
    if action == "ecrire":
        return argument                       # la dictée garde tout
    argument = POLITESSE.sub("", argument)
    if action in ("ouvrir_appli", "aller_sur"):
        return _premiere_proposition(argument) or argument
    return re.split(r"[.!?…]", argument, maxsplit=1)[0].strip() or argument   # chercher : 1re phrase


def trouver_action(texte):
    """Renvoie (action, argument) ou (None, None) si aucune commande ne correspond.
    argument vaut None pour les commandes simples ("copier")."""
    texte = _sans_hesitations(texte)
    action, argument = _analyser(texte)
    if action is None:
        # Phrase trop longue ou mélangée avec une conversation : on essaie le début seul.
        # "Enregistre, fais d'accord." -> "Enregistre". Uniquement le DÉBUT : une commande
        # au milieu d'une phrase est presque toujours quelqu'un qui parle, pas un ordre.
        debut = _premiere_proposition(texte)
        if debut and debut != texte.strip(" .!?"):
            action, argument = _analyser(debut)
    if argument is not None:
        argument = _nettoyer_argument(action, argument)
    return action, argument


def _analyser(texte):
    action, argument = _commande_a_argument(texte)
    if action:
        return action, argument

    texte_original = texte
    texte = nettoyer(texte)
    if not texte or len(texte.split()) > config.MOTS_MAX:
        return None, None

    # 1) Correspondance exacte : la phrase de commande apparaît dans le texte.
    #    On garde la plus longue ("coupe le son" doit gagner sur "coupe").
    trouvees = [
        (len(phrase), action, phrase)
        for action, phrases in PHRASES.items()
        for phrase in phrases
        if re.search(rf"(^|[ ']){re.escape(phrase)}( |$)", texte)   # "l'explorateur" contient "explorateur"
    ]
    if trouvees:
        _, action, _ = max(trouvees)
        return action, None

    # 2) Juste le nom d'une appli ou d'un site : "Spotify", "YouTube", "la calculatrice".
    #    Petites fautes tolérées ("Painte" -> Paint), mais plus strictement qu'après "ouvre" :
    #    sans mot-clé, "mineur" ne doit pas ouvrir le Minuteur ni "world" ouvrir Word.
    if (len(texte.split()) <= MOTS_MAX_NOM_SEUL
            and applis.trouver(texte_original, approx=True, ressemblance=RESSEMBLANCE_NOM_SEUL)):
        return "ouvrir_appli", texte_original

    # 3) Correspondance approchée sur tout le texte (faute de reconnaissance légère)
    meilleure, score_max = None, 0.0
    for action, phrases in PHRASES.items():
        for phrase in phrases:
            if len(phrase) < LONGUEUR_MIN_APPROX:
                continue
            score = difflib.SequenceMatcher(None, texte, phrase).ratio()
            if score > score_max:
                meilleure, score_max = action, score
    if score_max >= SIMILARITE_MIN:
        return meilleure, None
    return None, None


# Texte donné à Whisper pour l'orienter vers notre vocabulaire (en vrai français, avec accents).
# Limite : ~220 "tokens" (~120 mots) au total, au-delà Whisper ignore le début.
INDICE_WHISPER = (
    "Copier, coller, couper, annuler, rétablir, tout sélectionner, enregistrer, "
    "imprimer, gras, presse-papiers, espace, entrée, échap, effacer, "
    "supprimer, flèche du haut, flèche du bas, gauche, droite, début, fin, nouvel onglet, "
    "fermer l'onglet, actualiser, zoomer, dézoomer, plein écran, agrandir, "
    "réduire, bureau, explorateur, paramètres, gestionnaire des tâches, "
    "verrouiller, capture d'écran, monte le son, baisse le son, muet, pause, morceau suivant, "
    "diapo suivante, lance le diaporama. Ouvre Spotify. Va sur YouTube. Écris bonjour."
)
# (Pas de phrase d'exemple comme "Cherche une recette" : dans le bruit, Whisper la "recopie"
#  au lieu d'écouter, on l'a vu dans les essais : « C'est une recette, c'est une recette ».)
