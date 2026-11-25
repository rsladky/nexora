"""
Écoute du micro en continu et découpage en "phrases".

Principe : on mesure le volume toutes les 30 ms.
- volume > seuil  -> quelqu'un parle, on enregistre
- puis SILENCE_FIN secondes de calme -> la phrase est finie, on la renvoie
"""

import collections
import queue

import numpy as np
import sounddevice as sd

import config


def _volume(bloc):
    """Volume moyen d'un bloc audio (RMS)."""
    return float(np.sqrt(np.mean(bloc ** 2)))


class Micro:
    def __init__(self):
        self.taille_bloc = int(config.FREQUENCE * config.DUREE_BLOC)
        self.file = queue.Queue()
        self.flux = None
        self.seuil = config.SEUIL_MINIMUM

    def _recevoir(self, donnees, frames, temps, statut):
        # Appelé par sounddevice dans un autre thread à chaque bloc reçu
        self.file.put(donnees[:, 0].copy())

    def demarrer(self):
        self.flux = sd.InputStream(
            samplerate=config.FREQUENCE,
            channels=1,
            dtype="float32",
            blocksize=self.taille_bloc,
            callback=self._recevoir,
        )
        self.flux.start()

    def arreter(self):
        if self.flux is not None:
            self.flux.stop()
            self.flux.close()
            self.flux = None

    def calibrer(self):
        """Mesure le bruit de fond pendant DUREE_CALIBRAGE secondes (il faut se taire)."""
        nb_blocs = int(config.DUREE_CALIBRAGE / config.DUREE_BLOC)
        volumes = [_volume(self.file.get()) for _ in range(nb_blocs)]
        bruit = float(np.median(volumes))
        self.seuil = max(config.SEUIL_MINIMUM, bruit * config.MULTIPLICATEUR_BRUIT)
        return self.seuil

    def phrases(self):
        """Générateur : renvoie chaque phrase prononcée (tableau numpy 16 kHz)."""
        blocs_avant = collections.deque(maxlen=int(config.PRE_ENREGISTREMENT / config.DUREE_BLOC))
        blocs_silence_fin = int(config.SILENCE_FIN / config.DUREE_BLOC)
        blocs_max = int(config.DUREE_MAX / config.DUREE_BLOC)
        blocs_min = int(config.DUREE_MIN / config.DUREE_BLOC)

        # Volumes des 5 dernières secondes : le bruit de fond est remesuré en continu,
        # le seuil monte si la salle devient bruyante et redescend quand elle se calme.
        volumes_recents = collections.deque(maxlen=int(config.DUREE_MESURE_BRUIT / config.DUREE_BLOC))
        enregistrement = []
        silence = 0

        while self.flux is not None:
            try:
                bloc = self.file.get(timeout=0.5)
            except queue.Empty:
                continue
            volume = _volume(bloc)
            volumes_recents.append(volume)
            if not enregistrement and len(volumes_recents) == volumes_recents.maxlen:
                # 20e centile = le niveau des "creux" entre les mots = le bruit de fond.
                # Seuil figé pendant une phrase, pour ne pas la couper en plein milieu.
                bruit = float(np.percentile(volumes_recents, 20))
                self.seuil = max(config.SEUIL_MINIMUM, bruit * config.MULTIPLICATEUR_BRUIT)
            parle = volume > self.seuil

            if not enregistrement:
                blocs_avant.append(bloc)
                if parle:
                    enregistrement = list(blocs_avant)
                    silence = 0
                continue

            enregistrement.append(bloc)
            silence = 0 if parle else silence + 1

            if silence >= blocs_silence_fin or len(enregistrement) >= blocs_max:
                duree_parole = len(enregistrement) - silence
                if duree_parole >= blocs_min:
                    yield np.concatenate(enregistrement)
                enregistrement = []
                blocs_avant.clear()
                # On NE jette PAS le son arrivé pendant la transcription précédente :
                # une commande dite pendant ce temps serait perdue.
                # Sauf si le retard devient trop grand (salle qui parle sans arrêt) :
                # une commande exécutée 10 s après l'avoir dite surprendrait plus qu'autre chose.
                retard_max = int(config.RETARD_MAX / config.DUREE_BLOC)
                while self.file.qsize() > retard_max:
                    self.file.get_nowait()
