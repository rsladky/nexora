"""
Catalogue de tout ce qu'on peut ouvrir par son nom : applications et sites.

Trois sources, dans cet ordre de priorité :
1. APPLICATIONS : nos alias à la main ("calculette", "bloc notes"...) ;
2. les applis INSTALLÉES sur le PC, lues au démarrage dans la liste du menu Démarrer
   (Discord, Steam, Teams, jeux... rien à configurer, ça s'adapte à chaque PC) ;
3. SITES : si l'appli n'est pas installée, on ouvre le site ("Spotify" -> open.spotify.com).
"""

import difflib
import json
import os
import re
import subprocess
import threading
import webbrowser

from texte import compact, nettoyer

# Ce qu'on dit -> ce que Windows doit lancer (alias que la liste du menu Démarrer ne connaît pas)
APPLICATIONS = {
    "chrome": "chrome", "google chrome": "chrome",
    "firefox": "firefox", "edge": "msedge",
    "bloc notes": "notepad", "bloc note": "notepad", "notepad": "notepad",
    "calculatrice": "calc", "calculette": "calc",
    "paint": "mspaint",
    "word": "winword", "excel": "excel", "powerpoint": "powerpnt", "outlook": "outlook",
    "terminal": "wt", "invite de commande": "cmd", "invite de commandes": "cmd",
    "explorateur": "explorer", "explorateur de fichiers": "explorer", "mes fichiers": "explorer",
    "parametres": "ms-settings:", "reglages": "ms-settings:",
    "gestionnaire des taches": "taskmgr",
    "horloge": "ms-clock:", "minuteur": "ms-clock:", "alarme": "ms-clock:",
    "camera": "microsoft.windows.camera:",
    "panneau de configuration": "control",
}

SITES = {
    "youtube": "https://www.youtube.com",
    "google": "https://www.google.com",
    "gmail": "https://mail.google.com",
    "drive": "https://drive.google.com", "google drive": "https://drive.google.com",
    "maps": "https://www.google.com/maps", "google maps": "https://www.google.com/maps",
    "traducteur": "https://translate.google.com", "google traduction": "https://translate.google.com",
    "wikipedia": "https://fr.wikipedia.org",
    "github": "https://github.com",
    "chatgpt": "https://chatgpt.com", "chat gpt": "https://chatgpt.com",
    "claude": "https://claude.ai",
    "netflix": "https://www.netflix.com",
    "twitch": "https://www.twitch.tv",
    "amazon": "https://www.amazon.fr",
    "linkedin": "https://www.linkedin.com",
    "instagram": "https://www.instagram.com",
    "spotify": "https://open.spotify.com",
    "deezer": "https://www.deezer.com",
    "discord": "https://discord.com/app",
    "whatsapp": "https://web.whatsapp.com",
    "leboncoin": "https://www.leboncoin.fr", "le bon coin": "https://www.leboncoin.fr",
    "meteo": "https://meteofrance.com",
}

# Entrées du menu Démarrer qui ne sont pas de vraies applis
MOTS_EXCLUS = ("uninstall", "desinstall", "help", "aide", "documentation", "dokumentation",
               "readme", "read me", "faq", "release notes", "manuel", "manual", "licence",
               "license", "homepage", "website", "samples", "tools for", "support", "report a problem")
EXTENSIONS_EXCLUES = (".chm", ".html", ".htm", ".txt", ".pdf", ".url")

# Les noms de nos 3 listes, en version "compacte" pour comparer ce que Whisper écrit
_ALIAS = {compact(nom): cible for nom, cible in APPLICATIONS.items()}
_SITES = {compact(nom): url for nom, url in SITES.items()}
_installees = {}               # nom compact -> identifiant Windows de l'appli (AppID)
_chargement = threading.Event()


def _variantes(nom):
    """'Microsoft Teams' -> 'teams', 'PyCharm 2023.2.3' -> 'pycharm', 'Outlook (classic)' -> 'outlook',
    'Minecraft Launcher' -> 'minecraft', 'Docker Desktop' -> 'docker'."""
    nom = nettoyer(re.sub(r"\(.*?\)", "", nom))
    nom = re.sub(r"^(microsoft|google|navigateur) ", "", nom)
    nom = re.sub(r"( \d+)+$", "", nom)       # numéros de version à la fin
    nom = re.sub(r" (launcher|desktop|player|client|app|edition)$", "", nom)
    return nom


def _charger_applis_installees():
    """Lit la liste du menu Démarrer (Get-StartApps). ~2 s, fait en arrière-plan au démarrage."""
    try:
        resultat = subprocess.run(
            ["powershell", "-NoProfile", "-Command",
             "[Console]::OutputEncoding=[Text.Encoding]::UTF8; Get-StartApps | ConvertTo-Json -Compress"],
            capture_output=True, text=True, encoding="utf-8", timeout=30,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        liste = json.loads(resultat.stdout or "[]")
        if isinstance(liste, dict):            # une seule appli -> pas de liste
            liste = [liste]
    except (OSError, ValueError, subprocess.SubprocessError):
        liste = []

    noms, alias = {}, {}
    for appli in liste:
        nom, appid = appli.get("Name", ""), appli.get("AppID", "")
        n = nettoyer(nom)
        if (not n or appid.startswith("http") or appid.lower().endswith(EXTENSIONS_EXCLUES)
                or any(mot in n for mot in MOTS_EXCLUS)):
            continue
        noms.setdefault(compact(nom), appid)
        alias.setdefault(compact(_variantes(nom)), appid)
    for cle, appid in alias.items():
        noms.setdefault(cle, appid)            # le vrai nom gagne toujours sur un raccourci
    _installees.update({cle: appid for cle, appid in noms.items() if len(cle) >= 3})
    _chargement.set()


threading.Thread(target=_charger_applis_installees, daemon=True).start()


def trouver(nom, approx=False, ressemblance=0.85):
    """Cherche ce qu'il faut ouvrir pour ce nom.
    Renvoie ("alias", cible), ("installee", appid), ("site", url) ou None.
    approx=True tolère une petite faute de reconnaissance ("Painte" -> Paint) :
    il faut au moins `ressemblance` (0 à 1) entre ce qui est dit et le nom."""
    _chargement.wait(timeout=15)
    cle = compact(nom)
    if not cle:
        return None
    sources = (("alias", _ALIAS), ("installee", _installees), ("site", _SITES))
    for genre, dico in sources:
        if cle in dico:
            return genre, dico[cle]
    if approx and len(cle) >= 5:
        for genre, dico in sources:
            proches = difflib.get_close_matches(cle, dico.keys(), n=1, cutoff=ressemblance)
            if proches:
                return genre, dico[proches[0]]
    return None


def site(nom):
    """Adresse du site connu sous ce nom, ou None."""
    return _SITES.get(compact(nom))


def lancer(resultat):
    """Ouvre ce que trouver() a renvoyé."""
    genre, cible = resultat
    if genre == "site":
        webbrowser.open(cible)
    elif genre == "installee":
        # Ouvre l'appli comme un clic dans le menu Démarrer (marche aussi pour les applis du Store)
        subprocess.Popen(["explorer.exe", f"shell:AppsFolder\\{cible}"])
    else:
        os.startfile(cible)
