"""
Fonctions "pratiques" : ce qu'un simple raccourci clavier ne sait pas faire.
Ouvrir une application ou un site, chercher sur internet, écrire un texte dicté, verrouiller le PC.

Elles reçoivent un argument : ce qui a été dit après le mot-clé.
"ouvre Spotify" -> ouvrir("Spotify"), "écris bonjour" -> ecrire("bonjour").
"""

import ctypes
import re
import time
import urllib.parse
import webbrowser

import keyboard
import pyautogui

import applis
from texte import sans_articles

# Où chercher quand on dit "cherche X sur ..."
MOTEURS = {
    "youtube": "https://www.youtube.com/results?search_query={}",
    "wikipedia": "https://fr.wikipedia.org/w/index.php?search={}",
    "amazon": "https://www.amazon.fr/s?k={}",
    "google maps": "https://www.google.com/maps/search/{}",
    "maps": "https://www.google.com/maps/search/{}",
    "images": "https://www.google.com/search?tbm=isch&q={}",
    "google": "https://www.google.com/search?q={}",
}
MOTEUR_PAR_DEFAUT = MOTEURS["google"]


def _propre(texte):
    """Enlève la ponctuation que Whisper ajoute autour : ', Spotify.' -> 'Spotify'."""
    return texte.strip(" ,.;:!?")


def _trouver_debut(nom):
    """Cherche l'appli en tolérant des mots en trop à la fin :
    'Spotify tu vas voir' -> essaie 'Spotify tu vas voir', puis 'Spotify'."""
    mots = nom.split()
    for n in range(len(mots), 0, -1):
        trouve = applis.trouver(" ".join(mots[:n]), approx=True)
        if trouve:
            return trouve
    return None


def ouvrir(nom):
    """Ouvre une application installée, ou son site si elle ne l'est pas (voir applis.py)."""
    if not sans_articles(nom):
        return False
    trouve = _trouver_debut(nom)
    if trouve:
        try:
            applis.lancer(trouve)
            return True
        except OSError:
            pass  # pas trouvée par ce nom, on tente le menu Démarrer
    # Toujours rien : on fait comme un humain, touche Windows + nom + Entrée.
    pyautogui.press("win")
    time.sleep(0.6)
    keyboard.write(_propre(nom))
    time.sleep(1.0)
    pyautogui.press("enter")
    return True


def aller_sur(nom):
    """Ouvre un site : 'YouTube', 'le site de la SNCF', 'google point com'."""
    cle = sans_articles(nom)
    if not cle:
        return False
    if applis.site(cle):
        url = applis.site(cle)
    elif " point " in f" {cle} " or "." in cle:
        # "google point com" -> https://google.com
        url = "https://" + cle.replace(" point ", ".").replace(" ", "")
    else:
        # Site inconnu : DuckDuckGo "!ducky" envoie directement sur le premier résultat.
        url = "https://duckduckgo.com/?q=" + urllib.parse.quote("!ducky " + cle)
    webbrowser.open(url)
    return True


def chercher(requete):
    """Recherche sur internet. 'chats mignons sur YouTube' cherche sur YouTube, sinon Google."""
    requete = _propre(requete)
    modele = MOTEUR_PAR_DEFAUT
    for moteur, url in MOTEURS.items():
        # "chats mignons sur YouTube" -> on cherche "chats mignons" sur YouTube
        fin = re.compile(rf"\s+sur\s+{re.escape(moteur)}$", re.IGNORECASE)
        if fin.search(requete):
            modele = url
            requete = fin.sub("", requete)
            break
    requete = _propre(requete)
    if not requete:
        return False
    webbrowser.open(modele.format(urllib.parse.quote(requete)))
    return True


def ecrire(texte):
    """Tape le texte dicté là où se trouve le curseur (accents compris)."""
    texte = texte.strip(" ,")
    if not texte:
        return False
    keyboard.write(texte)
    return True


def verrouiller(_=None):
    """Verrouille la session. Win+L ne peut pas être simulé (Windows le bloque), on appelle Windows directement."""
    ctypes.windll.user32.LockWorkStation()
    return True
