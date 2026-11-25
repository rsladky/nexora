"""
Actions du logiciel — module PARTAGÉ entre la partie gestes et la partie voix.

Une action = un nom (ex. "copier") + ce qu'elle fait :
- soit un raccourci clavier (RACCOURCIS),
- soit une fonction Python (FONCTIONS), ex. ouvrir une application.
Les gestes et la voix appellent tous les deux executer("copier") :
on ne définit chaque action qu'une seule fois, ici.
"""

import signal
import threading
import time

import pyautogui

import fonctions

# Évite les petites pauses que pyautogui ajoute après chaque appel
pyautogui.PAUSE = 0

# nom de l'action -> touches à presser ensemble
RACCOURCIS = {
    # --- Édition ---
    "copier":            ["ctrl", "c"],
    "coller":            ["ctrl", "v"],
    "couper":            ["ctrl", "x"],
    "annuler":           ["ctrl", "z"],
    "retablir":          ["ctrl", "y"],
    "tout_selectionner": ["ctrl", "a"],
    "enregistrer":       ["ctrl", "s"],
    "rechercher":        ["ctrl", "f"],
    "nouveau":           ["ctrl", "n"],
    "ouvrir":            ["ctrl", "o"],
    "imprimer":          ["ctrl", "p"],
    "gras":              ["ctrl", "b"],
    "italique":          ["ctrl", "i"],
    "souligner":         ["ctrl", "u"],
    "presse_papier":     ["win", "v"],      # historique de tout ce qui a été copié

    # --- Touches ---
    "entree":            ["enter"],
    "echap":             ["esc"],
    "espace":            ["space"],
    "tabulation":        ["tab"],
    "supprimer":         ["delete"],        # efface le caractère APRÈS le curseur
    "effacer":           ["backspace"],     # efface le caractère AVANT le curseur
    "haut":              ["up"],
    "bas":               ["down"],
    "gauche":            ["left"],
    "droite":            ["right"],
    "debut":             ["home"],
    "fin":               ["end"],
    "monter":            ["pageup"],
    "descendre":         ["pagedown"],

    # --- Navigateur ---
    "nouvel_onglet":     ["ctrl", "t"],
    "fermer_onglet":     ["ctrl", "w"],
    "rouvrir_onglet":    ["ctrl", "shift", "t"],
    "onglet_suivant":    ["ctrl", "tab"],
    "onglet_precedent":  ["ctrl", "shift", "tab"],
    "page_precedente":   ["alt", "left"],
    "page_suivante":     ["alt", "right"],
    "actualiser":        ["f5"],
    "zoomer":            ["ctrl", "add"],       # + du pavé numérique, marche en AZERTY comme en QWERTY
    "dezoomer":          ["ctrl", "subtract"],
    "plein_ecran":       ["f11"],

    # --- Fenêtres et Windows ---
    "fermer_fenetre":    ["alt", "f4"],
    "changer_fenetre":   ["alt", "tab"],
    "agrandir":          ["win", "up"],
    "reduire":           ["win", "down"],
    "fenetre_gauche":    ["win", "left"],
    "fenetre_droite":    ["win", "right"],
    "bureau":            ["win", "d"],
    "menu_demarrer":     ["win"],
    "explorateur":       ["win", "e"],
    "parametres":        ["win", "i"],
    "gestionnaire_taches": ["ctrl", "shift", "esc"],
    "capture_ecran":     ["win", "shift", "s"],

    # --- Présentation (PowerPoint, Google Slides, PDF en plein écran) ---
    "lancer_diaporama":  ["f5"],
    "diapo_suivante":    ["right"],
    "diapo_precedente":  ["left"],
    "quitter_diaporama": ["esc"],
    "ecran_noir":        ["b"],             # PowerPoint : écran noir, re-dire pour revenir

    # --- Son et musique ---
    "volume_plus":       ["volumeup"],
    "volume_moins":      ["volumedown"],
    "muet":              ["volumemute"],
    "lecture_pause":     ["playpause"],
    "morceau_suivant":   ["nexttrack"],
    "morceau_precedent": ["prevtrack"],
}


# nom de l'action -> fonction appelée avec ce qui a été dit après le mot-clé
FONCTIONS = {
    "ouvrir_appli": fonctions.ouvrir,       # "ouvre Spotify"
    "aller_sur":    fonctions.aller_sur,    # "va sur YouTube"
    "chercher":     fonctions.chercher,     # "cherche recette crêpes"
    "ecrire":       fonctions.ecrire,       # "écris bonjour à tous"
    "verrouiller":  fonctions.verrouiller,  # Win+L est bloqué par Windows, d'où une fonction
}


def executer(action, argument=None):
    """Lance l'action. Renvoie False si l'action est inconnue."""
    global _dernier_ctrl_c
    if action in FONCTIONS:
        return FONCTIONS[action](argument)
    touches = RACCOURCIS.get(action)
    if touches is None:
        return False
    if touches == ["ctrl", "c"]:
        _dernier_ctrl_c = time.time()
    pyautogui.hotkey(*touches)
    return True


# --- Protection Ctrl+C ---
# Si la fenêtre active est le terminal qui fait tourner le programme, Ctrl+C l'arrête net
# (c'est le raccourci "arrêter le programme" des terminaux). Quand c'est NOUS qui venons
# d'envoyer Ctrl+C (commande "copier" ou geste), on ignore l'arrêt. Un vrai Ctrl+C de
# l'utilisateur au clavier arrête toujours le programme.
_dernier_ctrl_c = 0.0


def _sur_ctrl_c(signum, frame):
    if time.time() - _dernier_ctrl_c < 2.0:
        return
    raise KeyboardInterrupt


# signal ne peut être configuré que depuis le thread principal (le cas quand on fait "import actions")
if threading.current_thread() is threading.main_thread():
    signal.signal(signal.SIGINT, _sur_ctrl_c)
