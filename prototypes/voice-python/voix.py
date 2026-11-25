"""
Reconnaissance du LOCUTEUR : est-ce bien l'utilisateur enregistré qui parle ?

Principe :
- un modèle (CAM++) transforme quelques secondes de voix en "empreinte" : 192 nombres
  qui décrivent le timbre de la voix, quels que soient les mots prononcés ;
- à l'inscription, on fait la moyenne des empreintes de plusieurs phrases = le profil ;
- ensuite, pour chaque phrase entendue, on compare son empreinte au profil.
  Score proche de 1 = même voix, proche de 0 = voix différente.

Ce n'est pas un système de sécurité : un enregistrement de ta voix passerait.
C'est un filtre pour que les gens autour ne déclenchent pas de commandes.
"""

import os
import urllib.request

import numpy as np
import sherpa_onnx

import config

DOSSIER = os.path.dirname(os.path.abspath(__file__))
FICHIER_MODELE = os.path.join(DOSSIER, "modeles", "campplus_voxceleb.onnx")
URL_MODELE = ("https://github.com/k2-fsa/sherpa-onnx/releases/download/"
              "speaker-recongition-models/3dspeaker_speech_campplus_sv_en_voxceleb_16k.onnx")
FICHIER_PROFIL = os.path.join(DOSSIER, "profil_voix.npz")


def _telecharger_modele():
    if not os.path.exists(FICHIER_MODELE):
        print("Téléchargement du modèle de reconnaissance de voix (~30 Mo, une seule fois)...")
        os.makedirs(os.path.dirname(FICHIER_MODELE), exist_ok=True)
        urllib.request.urlretrieve(URL_MODELE, FICHIER_MODELE + ".part")
        os.replace(FICHIER_MODELE + ".part", FICHIER_MODELE)


class VerificateurVoix:
    def __init__(self):
        _telecharger_modele()
        self.extracteur = sherpa_onnx.SpeakerEmbeddingExtractor(
            sherpa_onnx.SpeakerEmbeddingExtractorConfig(model=FICHIER_MODELE, num_threads=2))
        self.profil = None
        self.seuil = config.SEUIL_VOIX
        self.actif = True          # False = on accepte toutes les voix même si un profil existe
        self.nb_phrases = 0
        self.charger()

    # --- Empreintes ---

    def empreinte(self, audio):
        """audio (16 kHz) -> vecteur de longueur 1 qui résume la voix."""
        flux = self.extracteur.create_stream()
        flux.accept_waveform(config.FREQUENCE, audio)
        flux.input_finished()
        e = np.array(self.extracteur.compute(flux))
        return e / np.linalg.norm(e)

    def score(self, audio):
        """Ressemblance avec le profil : ~0.6-0.8 pour l'utilisateur, < 0.45 pour les autres."""
        return float(self.empreinte(audio) @ self.profil)

    def filtre_actif(self):
        return self.actif and self.profil is not None

    def est_utilisateur(self, audio):
        """Renvoie (accepté ?, score). Sans profil ou filtre désactivé, tout le monde est accepté."""
        if not self.filtre_actif():
            return True, None
        s = self.score(audio)
        return s >= self.seuil, s

    # --- Profil ---

    def creer_profil(self, audios):
        """Crée le profil à partir de plusieurs enregistrements de l'utilisateur, et le sauvegarde."""
        moyenne = np.mean([self.empreinte(a) for a in audios], axis=0)
        self.profil = moyenne / np.linalg.norm(moyenne)
        self.nb_phrases = len(audios)
        self.sauvegarder()

    def sauvegarder(self):
        """Écrit le profil ET les réglages (seuil, actif)."""
        if self.profil is None:
            return
        np.savez(FICHIER_PROFIL, profil=self.profil, seuil=self.seuil,
                 actif=self.actif, nb_phrases=self.nb_phrases)

    def charger(self):
        if not os.path.exists(FICHIER_PROFIL):
            return
        donnees = np.load(FICHIER_PROFIL)
        self.profil = donnees["profil"]
        self.seuil = float(donnees["seuil"])
        self.actif = bool(donnees["actif"])
        self.nb_phrases = int(donnees["nb_phrases"])

    def supprimer_profil(self):
        self.profil = None
        if os.path.exists(FICHIER_PROFIL):
            os.remove(FICHIER_PROFIL)
