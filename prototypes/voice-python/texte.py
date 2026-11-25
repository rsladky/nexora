"""Petits outils pour comparer du texte reconnu par Whisper avec nos listes."""

import re
import unicodedata

MOTS_INUTILES = {"le", "la", "les", "l", "un", "une", "des", "du", "de", "d",
                 "sur", "site", "application", "appli", "logiciel"}


def nettoyer(texte):
    """'  Copié ! ' -> 'copie' : minuscules, sans accents ni ponctuation."""
    texte = unicodedata.normalize("NFD", texte.lower())
    texte = "".join(c for c in texte if unicodedata.category(c) != "Mn")
    texte = texte.replace("’", "'")
    texte = re.sub(r"[^a-z0-9' ]+", " ", texte)
    return " ".join(texte.split())


def sans_articles(texte):
    """"l'application Spotify." -> 'spotify' (pour comparer avec nos listes)."""
    mots = nettoyer(texte).replace("'", " ").split()
    while mots and mots[0] in MOTS_INUTILES:
        mots.pop(0)
    return " ".join(mots)


def compact(texte):
    """'Bloc-notes', 'What's App', 'whatsapp' -> 'blocnotes', 'whatsapp', 'whatsapp'.
    Whisper n'écrit pas toujours les noms de marque pareil : on compare sans espaces ni tirets."""
    return re.sub(r"[ ']", "", sans_articles(texte))
