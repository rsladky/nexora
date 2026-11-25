"""
Assistant vocal : micro -> Whisper -> commande -> raccourci clavier.

Utilisation seule :         python main.py
Depuis le programme gestes : voir l'exemple dans le README (AssistantVocal().demarrer()).
"""

import threading

import actions
import commandes
from micro import Micro
from transcription import Transcripteur
from voix import VerificateurVoix


class AssistantVocal:
    def __init__(self, modele=None, simulation=False, afficher=print):
        """
        simulation : True = affiche les actions sans appuyer sur les touches (pour tester)
        afficher   : fonction appelée pour les messages (print par défaut, ou une fonction de l'interface)
        """
        self.afficher = afficher
        self.simulation = simulation
        self.afficher("Chargement du modèle Whisper (peut prendre un moment la 1re fois)...")
        self.transcripteur = Transcripteur(modele) if modele else Transcripteur()
        self.verif_voix = VerificateurVoix()
        if self.verif_voix.filtre_actif():
            self.afficher(f"Voix enregistrée : les autres voix seront ignorées (sévérité {self.verif_voix.seuil:.2f}).")
        else:
            self.afficher("Aucune voix enregistrée (ou filtre désactivé) : tout le monde peut donner des commandes."
                          " Pour enregistrer ta voix : python interface_voix.py")
        self.micro = Micro()
        self.en_pause = False
        self.thread = None

    def demarrer(self):
        """Lance l'écoute dans un thread en arrière-plan (ne bloque pas le programme)."""
        self.thread = threading.Thread(target=self.boucle, daemon=True)
        self.thread.start()

    def arreter(self):
        self.micro.arreter()

    def boucle(self):
        self.micro.demarrer()
        self.afficher("Calibrage du bruit de fond, silence 1 seconde...")
        seuil = self.micro.calibrer()
        self.afficher(f"Prêt ! (seuil de volume : {seuil:.3f}) Dis une commande, ex. « copier ».")

        for audio in self.micro.phrases():
            # On vérifie la voix AVANT Whisper : une voix inconnue ne coûte même pas de transcription
            accepte, score = self.verif_voix.est_utilisateur(audio)
            if not accepte:
                self.afficher(f"(voix inconnue, ressemblance {score:.2f} < {self.verif_voix.seuil:.2f} : ignoré)")
                continue
            texte = self.transcripteur.transcrire(audio)
            if not texte:
                continue
            action, argument = commandes.trouver_action(texte)
            self.traiter(texte, action, argument)

    def traiter(self, texte, action, argument=None):
        if action == commandes.PAUSE_ECOUTE:
            self.en_pause = True
            self.afficher(f"« {texte} » -> écoute en pause (dis « reprends l'écoute »)")
        elif action == commandes.REPRISE_ECOUTE:
            self.en_pause = False
            self.afficher(f"« {texte} » -> écoute reprise")
        elif self.en_pause:
            self.afficher(f"« {texte} » (en pause, ignoré)")
        elif action is None:
            self.afficher(f"« {texte} » -> aucune commande")
        else:
            if not self.simulation:
                actions.executer(action, argument)
            detail = f"« {argument} »" if argument else actions.RACCOURCIS.get(action, "")
            self.afficher(f"« {texte} » -> {action} {detail}"
                          + (" [simulation]" if self.simulation else ""))
