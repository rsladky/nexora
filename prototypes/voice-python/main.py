"""
Lance la reconnaissance vocale seule.

    python main.py               -> écoute et appuie vraiment sur les touches
    python main.py --simulation  -> écoute et affiche seulement ce qu'il ferait
    python main.py --modele base -> modèle plus léger/rapide (moins précis)
"""

import argparse

from assistant_vocal import AssistantVocal

parser = argparse.ArgumentParser(description="Pilotage du PC à la voix")
parser.add_argument("--simulation", action="store_true", help="n'appuie sur aucune touche")
parser.add_argument("--modele", help="tiny, base, small, medium...")
args = parser.parse_args()

assistant = AssistantVocal(modele=args.modele, simulation=args.simulation)
try:
    assistant.boucle()   # ici on écoute directement, sans thread
except KeyboardInterrupt:
    print("\nArrêt.")
finally:
    assistant.arreter()
