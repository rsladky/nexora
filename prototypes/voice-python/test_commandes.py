"""Tests de la correspondance texte -> action :  python test_commandes.py"""

from commandes import trouver_action

CAS = [
    ("Copier.", "copier"),
    (" copié ! ", "copier"),
    ("Colle", "coller"),
    ("Annule.", "annuler"),
    ("Tout sélectionner", "tout_selectionner"),
    ("Monte le son", "volume_plus"),       # doit gagner sur "monte" (page précédente)
    ("Coupe le son.", "muet"),             # doit gagner sur "coupe" (couper)
    ("Ferme l'onglet", "fermer_onglet"),
    ("Capture d'écran", "capture_ecran"),
    ("Arrête l'écoute.", "pause_ecoute"),
    ("Copie", "copier"),
    ("Coppier", "copier"),                 # faute de reconnaissance -> correspondance approchée
    ("Je vais copier ce texte dans le document", None),  # phrase trop longue = conversation
    ("Bonjour", None),
    ("", None),
    # Nouvelles commandes
    ("Espace.", "espace"),
    ("Efface !", "effacer"),
    ("Supprime.", "supprimer"),
    ("Mets en gras.", "gras"),
    ("Imprime.", "imprimer"),
    ("Ouvre l'explorateur", "explorateur"),   # doit gagner sur "ouvre"
    ("Ouvre les paramètres", "parametres"),
    ("Rouvre l'onglet", "rouvrir_onglet"),
    ("Retour.", "page_precedente"),
    ("Page suivante", "page_suivante"),
    ("Actualise", "actualiser"),
    ("Zoom arrière", "dezoomer"),
    ("Plein écran", "plein_ecran"),
    ("Fenêtre à gauche", "fenetre_gauche"),  # doit gagner sur "gauche"
    ("Flèche du bas", "bas"),
    ("Gestionnaire des tâches", "gestionnaire_taches"),
    ("Presse-papiers", "presse_papier"),
    ("Musique suivante", "morceau_suivant"),
    ("Baisse le son", "volume_moins"),        # ne doit pas déclencher "bas"
    ("Fini.", None),                          # mot court : pas de correspondance approchée avec "fin"
    ("Verrouiller.", "verrouiller"),
    ("Diapo suivante", "diapo_suivante"),
    ("Lance le diaporama.", "lancer_diaporama"),  # "lance" est un mot-clé, mais c'est une commande connue
    ("Fin du diaporama", "quitter_diaporama"),
    ("Recherche", "rechercher"),              # seul = Ctrl+F ; avec un argument = recherche internet
    ("Ouvre", "ouvrir"),                      # seul = Ctrl+O
    ("Ouvre les paramètres.", "parametres"),
    ("Ouvre un nouvel onglet", "nouvel_onglet"),
]

# Commandes avec argument : (texte, action attendue, argument attendu)
CAS_ARGUMENT = [
    ("Ouvre Spotify.", "ouvrir_appli", "Spotify"),
    ("Lance la calculatrice", "ouvrir_appli", "la calculatrice"),
    ("Va sur YouTube.", "aller_sur", "YouTube"),
    ("Cherche recette de crêpes.", "chercher", "recette de crêpes"),
    ("Cherche comment copier un fichier", "chercher", "comment copier un fichier"),  # pas Ctrl+C !
    ("Écris, Bonjour à tous.", "ecrire", "Bonjour à tous."),
    ("Écris copier", "ecrire", "copier"),     # la dictée écrit le mot, ne fait pas Ctrl+C
    ("Tape je serai en retard de dix minutes, désolé.", "ecrire",
     "je serai en retard de dix minutes, désolé."),
    # Juste le nom d'une appli ou d'un site (applis.py) : on l'ouvre
    ("YouTube.", "ouvrir_appli", "YouTube"),
    ("Calculatrice.", "ouvrir_appli", "Calculatrice"),
    ("La calculatrice", "ouvrir_appli", "La calculatrice"),
    ("Spotify", "ouvrir_appli", "Spotify"),
    ("Bloc-notes", "ouvrir_appli", "Bloc-notes"),
    ("What's App ?", "ouvrir_appli", "What's App"),
    ("Painte.", "ouvrir_appli", "Painte"),             # faute de Whisper sur "Paint"
    ("Terminale.", "ouvrir_appli", "Terminale"),       # faute de Whisper sur "Terminal"
    ("Chromes", "ouvrir_appli", "Chromes"),
    ("Ouvre la calculatrisse", "ouvrir_appli", "la calculatrisse"),  # après "ouvre", plus tolérant
    ("Mineur", None, None),                            # ne doit pas ouvrir le Minuteur
    ("World", None, None),                             # ne doit pas ouvrir Word
    # Vraies phrases relevées en salle de cours : commande + hésitation ou conversation autour
    ("Va sur YouTube, tu vas voir.", "aller_sur", "YouTube"),
    ("Euh, ouvre Spotify s'il te plaît.", "ouvrir_appli", "Spotify"),
    ("Enregistre, fais d'accord.", "enregistrer", None),
    ("Bon, ok, copier.", "copier", None),
    ("Allez sur YouTube", "aller_sur", "YouTube"),
    # Vraies conversations du terminal : rien ne doit se déclencher
    ("mais qu'il est capable de s'ilver, imprimer, qu'il est capable de s'ilver, imprimer, c'est ça ?", None, None),
    ("je fais ajouter ou Spotify,", None, None),
    ("La notion que c'est une simulation, apparemment, restart, abonne.", None, None),
    ("Merci d'avoir regardé l'onglet.", None, None),
    ("C'est bon, c'est bon.", None, None),
    ("Non, non, non.", None, None),
    ("Au revoir.", None, None),
    ("Allons-y !", None, None),
    ("Il ne faut pas me réveiller, il ne faut pas me réveiller,", None, None),
]

# Cohérence : chaque commande vocale existe dans actions.py, et aucune phrase ne dépasse MOTS_MAX
import actions, commandes, config
for action, phrases in commandes.PHRASES.items():
    if action not in (commandes.PAUSE_ECOUTE, commandes.REPRISE_ECOUTE):
        assert action in actions.RACCOURCIS or action in actions.FONCTIONS, \
            f"{action} n'existe pas dans actions.py"
    for p in phrases:
        assert len(p.split()) <= config.MOTS_MAX, f"« {p} » est trop longue (MOTS_MAX)"
for action in commandes.MOTS_CLES:
    assert action in actions.FONCTIONS, f"{action} n'a pas de fonction dans actions.py"

erreurs = 0
for texte, attendu in CAS:
    obtenu, _ = trouver_action(texte)
    ok = obtenu == attendu
    erreurs += not ok
    print(f"{'OK ' if ok else 'ERR'} {texte!r:45} -> {obtenu} (attendu {attendu})")
for texte, attendu, arg_attendu in CAS_ARGUMENT:
    obtenu, argument = trouver_action(texte)
    ok = (obtenu, argument) == (attendu, arg_attendu)
    erreurs += not ok
    print(f"{'OK ' if ok else 'ERR'} {texte!r:45} -> {obtenu}({argument!r})")
CAS += CAS_ARGUMENT
print(f"\n{len(CAS) - erreurs}/{len(CAS)} réussis")
