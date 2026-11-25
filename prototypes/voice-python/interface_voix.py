"""
Petite interface pour enregistrer SA voix, la tester et régler la sévérité.

    python interface_voix.py

Une fois la voix enregistrée, main.py ignore les commandes dites par d'autres personnes.
"""

import queue
import threading
import tkinter as tk
from tkinter import messagebox, ttk

import numpy as np

from micro import Micro
from voix import VerificateurVoix

# Phrases à lire pour l'inscription : des phrases longues (timbre de la voix)
# et des commandes courtes (c'est ce qu'il faudra reconnaître ensuite).
PHRASES_INSCRIPTION = [
    "Bonjour, je configure la reconnaissance de ma voix.",
    "Copier, coller, annuler, enregistrer.",
    "Ouvre Spotify et va sur YouTube.",
    "Le soleil se lève tôt en été à Marseille.",
    "Monte le son, baisse le son, diapo suivante.",
    "Cherche une recette de crêpes facile.",
]


class Interface:
    def __init__(self, fenetre):
        self.fenetre = fenetre
        self.verif = VerificateurVoix()
        self.micro = None
        self.messages = queue.Queue()   # les threads micro parlent à l'interface par ici

        fenetre.title("VocaReco — Ma voix")
        fenetre.geometry("520x560")
        fenetre.resizable(False, True)
        marge = {"padx": 12, "pady": 6}

        self.statut = ttk.Label(fenetre, font=("Segoe UI", 11, "bold"))
        self.statut.pack(anchor="w", **marge)

        # 1. Enregistrement
        cadre = ttk.LabelFrame(fenetre, text="1. Enregistrer ma voix")
        cadre.pack(fill="x", **marge)
        self.bouton_inscription = ttk.Button(cadre, text="Enregistrer ma voix (6 phrases)",
                                             command=self.lancer_inscription)
        self.bouton_inscription.pack(anchor="w", padx=8, pady=6)
        self.progression = ttk.Label(cadre, text="")
        self.progression.pack(anchor="w", padx=8)
        self.phrase = tk.Label(cadre, text="", font=("Segoe UI", 14), wraplength=470,
                               justify="left", fg="#1a5fb4")
        self.phrase.pack(anchor="w", padx=8, pady=(2, 8))

        # 2. Test
        cadre = ttk.LabelFrame(fenetre, text="2. Tester")
        cadre.pack(fill="both", expand=True, **marge)
        ttk.Label(cadre, text="Parle, puis fais parler quelqu'un d'autre : sa voix doit être ignorée.",
                  wraplength=470).pack(anchor="w", padx=8, pady=(6, 0))
        self.bouton_test = ttk.Button(cadre, text="Commencer le test", command=self.basculer_test)
        self.bouton_test.pack(anchor="w", padx=8, pady=6)
        self.journal = tk.Listbox(cadre, height=8, font=("Consolas", 10))
        self.journal.pack(fill="both", expand=True, padx=8, pady=(0, 8))

        # 3. Réglages
        cadre = ttk.LabelFrame(fenetre, text="3. Réglages")
        cadre.pack(fill="x", **marge)
        self.actif = tk.BooleanVar(value=self.verif.actif)
        ttk.Checkbutton(cadre, text="Ignorer les commandes des autres voix", variable=self.actif,
                        command=self.changer_reglages).pack(anchor="w", padx=8, pady=(6, 0))
        ligne = ttk.Frame(cadre)
        ligne.pack(fill="x", padx=8, pady=6)
        ttk.Label(ligne, text="Sévérité :").pack(side="left")
        self.seuil = tk.DoubleVar(value=self.verif.seuil)
        ttk.Scale(ligne, from_=0.30, to=0.80, variable=self.seuil,
                  command=lambda _: self.afficher_seuil()).pack(side="left", fill="x", expand=True, padx=6)
        self.texte_seuil = ttk.Label(ligne, width=5)
        self.texte_seuil.pack(side="left")
        ttk.Label(cadre, text="Plus haut = les autres passent moins, mais tu risques d'être refusé plus souvent.",
                  wraplength=470, foreground="#666").pack(anchor="w", padx=8, pady=(0, 6))
        self.seuil.trace_add("write", lambda *_: self.changer_reglages())

        ttk.Button(fenetre, text="Supprimer ma voix", command=self.supprimer).pack(anchor="e", **marge)

        if self.verif.profil is not None:
            self.bouton_inscription.config(text="Ré-enregistrer ma voix")
        self.afficher_seuil()
        self.maj_statut()
        self.lire_messages()
        fenetre.protocol("WM_DELETE_WINDOW", self.fermer)

    # --- Affichage ---

    def maj_statut(self):
        if self.verif.profil is None:
            self.statut.config(text="Aucune voix enregistrée : tout le monde peut donner des commandes.",
                               foreground="#c01c28")
        elif not self.verif.actif:
            self.statut.config(text="Voix enregistrée, mais le filtre est désactivé.", foreground="#e5a50a")
        else:
            self.statut.config(text=f"Voix enregistrée ({self.verif.nb_phrases} phrases) : "
                                    "les autres voix sont ignorées.", foreground="#26a269")

    def afficher_seuil(self):
        self.texte_seuil.config(text=f"{self.seuil.get():.2f}")

    def lire_messages(self):
        """Toutes les 100 ms, applique ce que les threads micro ont demandé d'afficher."""
        while not self.messages.empty():
            fonction, args = self.messages.get()
            fonction(*args)
        self.fenetre.after(100, self.lire_messages)

    def envoyer(self, fonction, *args):
        """Depuis un thread : demande à l'interface d'exécuter fonction(*args)."""
        self.messages.put((fonction, args))

    # --- Micro ---

    def demarrer_micro(self):
        self.micro = Micro()
        self.micro.demarrer()
        self.micro.calibrer()
        return self.micro.phrases()

    def arreter_micro(self):
        if self.micro:
            self.micro.arreter()
            self.micro = None

    # --- Inscription ---

    def lancer_inscription(self):
        self.arreter_test()
        self.bouton_inscription.config(state="disabled")
        self.bouton_test.config(state="disabled")
        threading.Thread(target=self.inscription, daemon=True).start()

    def inscription(self):
        self.envoyer(self.progression.config, {"text": "Silence 1 seconde (mesure du bruit)..."})
        self.envoyer(self.phrase.config, {"text": ""})
        phrases = self.demarrer_micro()
        audios = []
        try:
            for i, texte in enumerate(PHRASES_INSCRIPTION, 1):
                self.envoyer(self.progression.config,
                             {"text": f"Phrase {i}/{len(PHRASES_INSCRIPTION)} — lis à voix haute, normalement :"})
                self.envoyer(self.phrase.config, {"text": f"« {texte} »"})
                audios.append(next(phrases))
        except StopIteration:
            return
        finally:
            self.arreter_micro()
        self.verif.creer_profil(audios)
        # Contrôle : chaque phrase enregistrée doit ressembler au profil
        scores = [self.verif.score(a) for a in audios]
        self.envoyer(self.fin_inscription, min(scores), max(scores))

    def fin_inscription(self, mini, maxi):
        self.progression.config(text=f"Voix enregistrée ! Ressemblance de tes phrases : {mini:.2f} à {maxi:.2f}")
        self.phrase.config(text="Tu peux maintenant tester ci-dessous.")
        self.bouton_inscription.config(state="normal", text="Ré-enregistrer ma voix")
        self.bouton_test.config(state="normal")
        self.maj_statut()

    # --- Test ---

    def basculer_test(self):
        if self.micro:
            self.arreter_test()
        elif self.verif.profil is None:
            messagebox.showinfo("VocaReco", "Enregistre d'abord ta voix (étape 1).")
        else:
            self.bouton_test.config(text="Arrêter le test")
            self.bouton_inscription.config(state="disabled")
            self.journal.insert(0, "Silence 1 seconde (mesure du bruit)...")
            threading.Thread(target=self.test, daemon=True).start()

    def arreter_test(self):
        self.arreter_micro()
        self.bouton_test.config(text="Commencer le test")
        self.bouton_inscription.config(state="normal")

    def test(self):
        phrases = self.demarrer_micro()
        self.envoyer(self.journal.insert, 0, "Prêt, parle !")
        for audio in phrases:
            s = self.verif.score(audio)
            duree = len(audio) / 16000
            if s >= self.verif.seuil:
                ligne = f"✅ {s:.2f}  accepté   ({duree:.1f} s)"
            else:
                ligne = f"❌ {s:.2f}  ignoré    ({duree:.1f} s)"
            self.envoyer(self.journal.insert, 0, ligne)

    # --- Réglages ---

    def changer_reglages(self):
        self.verif.seuil = round(float(np.clip(self.seuil.get(), 0.3, 0.8)), 2)
        self.verif.actif = self.actif.get()
        self.verif.sauvegarder()
        self.maj_statut()

    def supprimer(self):
        if self.verif.profil is None:
            return
        if messagebox.askyesno("VocaReco", "Supprimer ta voix ? Tout le monde pourra de nouveau donner des commandes."):
            self.arreter_test()
            self.verif.supprimer_profil()
            self.bouton_inscription.config(text="Enregistrer ma voix (6 phrases)")
            self.maj_statut()

    def fermer(self):
        self.arreter_micro()
        self.fenetre.destroy()


if __name__ == "__main__":
    fenetre = tk.Tk()
    Interface(fenetre)
    fenetre.mainloop()
