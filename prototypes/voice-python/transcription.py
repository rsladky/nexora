"""Transformation de l'audio en texte avec Whisper (hors ligne, via faster-whisper)."""

import re

from faster_whisper import WhisperModel

import commandes
import config

# Si Whisper n'est pas sûr de lui, on préfère ne rien faire plutôt qu'appuyer sur une mauvaise touche.
CONFIANCE_MIN = -1.2     # log-probabilité moyenne minimale d'un segment
PROBA_SILENCE_MAX = 0.6   # au-dessus, Whisper pense que c'était du bruit et pas de la parole

# Whisper a appris sur des vidéos YouTube sous-titrées : sur du bruit ou un son mal compris,
# il "invente" souvent ces phrases de fin de vidéo. On les enlève du texte.
HALLUCINATIONS = re.compile(
    r"merci d'avoir regardé( cette vidéo)?|sous-titr\w+.*$|"
    r"abonnez-vous[^.!?]*|n'oubliez pas de vous abonner[^.!?]*|"
    r"merci (à tous|beaucoup)? ?(de|pour) (votre|vos) (attention|écoute)",
    re.IGNORECASE)


class Transcripteur:
    def __init__(self, modele=config.MODELE_WHISPER):
        # int8 = version allégée du modèle, beaucoup plus rapide sur processeur.
        # Le modèle est téléchargé automatiquement la première fois (~140 Mo pour "base").
        self.modele = WhisperModel(modele, device="cpu", compute_type="int8")
        # On donne à Whisper la liste des commandes : il les reconnaîtra mieux
        # ("colle" au lieu de rien, "capture d'écran" au lieu de "capture des crans").
        self.indice = commandes.INDICE_WHISPER

    def transcrire(self, audio):
        """audio : tableau numpy float32 en 16 kHz mono -> texte reconnu (peut être vide)."""
        segments, _ = self.modele.transcribe(
            audio,
            language=config.LANGUE,
            beam_size=1,                      # plus rapide, suffisant pour des mots courts
            condition_on_previous_text=False,
            without_timestamps=True,
            initial_prompt=self.indice,
        )
        morceaux = [
            s.text
            for s in segments
            if s.avg_logprob > CONFIANCE_MIN and s.no_speech_prob < PROBA_SILENCE_MAX
        ]
        return HALLUCINATIONS.sub("", " ".join(morceaux)).strip(" ,")
