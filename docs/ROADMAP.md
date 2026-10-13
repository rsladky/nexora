# Roadmap — YDAYS semaine C

| Séance | Date | Objectif | Livrable |
|---|---|---|---|
| YDAYS 1 | 14/10/2026 | Staffing, briefing | équipe constituée, repo prêt |
| YDAYS 2 | 25/11/2026 | Finalisation équipes, recherche & conception | catalogue gestes/voix validé, POC en cours |
| YDAYS 3 | 16/12/2026 | **Oral de validation** | démo POC, architecture, maquette UI |
| YDAYS 4-5 | 20-21/01/2027 | Développement gestes & voix | classifieur ML, grammaire voix étendue |
| YDAYS 6 | 03/03/2027 | Intégration dans l'app | UI + édition des mappings |
| YDAYS 7 | 24/03/2027 | Tests fonctionnels | premiers tests utilisateurs |
| YDAYS 8 | 14/04/2027 | Optimisation | latence, CPU, faux positifs |
| YDAYS 9 | 05/05/2027 | Finalisation | packaging .msi + AppImage |
| YDAYS 10 | 26/05/2027 | Recette, préparation oral | rapport de tests |
| YDAYS 11 | 27/05/2027 | **Oral final** | démo + présentation |

## Répartition par rôle

- **Data IA (×3)** : suivi des mains (ONNX), classifieur de gestes, reconnaissance vocale, précision/fiabilité.
- **Dev (×2)** : UI Tauri/React, contrôle curseur/clics/raccourcis, personnalisation, site vitrine (repo séparé).
- **Infra (×1)** : CI/CD, packaging, optimisation des ressources IA, sécurité.
- **Marketing (×1)** : cibles, concurrence, identité, modèle économique, étude de marché.

## Avant le 16/12 (priorités)

1. Installer les dépendances système, vérifier `cargo test`.
2. Scaffolder `app/` (Tauri + React).
3. POC : l'index déplace le curseur, pincement = clic, « copie »/« colle » en voix.
4. Définir le protocole de tests utilisateurs.
