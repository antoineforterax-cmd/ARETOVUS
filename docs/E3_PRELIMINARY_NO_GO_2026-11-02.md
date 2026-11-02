# E3 — décision préliminaire NO-GO, ARETOVUS expérimental

**Relevé : 2 novembre 2026.** **Décision : NO-GO pour extension du testnet à des participants publics ou des fonds réels.** Ce document ne déclenche ni déploiement, ni release, ni modification du protocole. Le testnet de laboratoire sur machines et réseaux contrôlés peut continuer avec comptes jetables.

## Périmètre et autorité

Le responsable a demandé de poursuivre jusqu'à E3 en différant **E1 (audit indépendant)**. Cette décision de calendrier ne constitue pas l'audit : E1 reste **non réalisée et non validée**. Une éventuelle ouverture d'un testnet public, a fortiori un mainnet, devra faire l'objet d'une décision différente sur des preuves nouvelles. **Aucun fonds réel.**

## Tableau des critères, preuves vérifiables et lacunes

| Porte | État vérifié au relevé | Conséquence E3 |
| --- | --- | --- |
| A1–A4, B1–B4 | Intégrés précédemment ; B4 final via [#164](https://github.com/antoineforterax-cmd/ARETOVUS/pull/164) et [CI intégrée #41462436211](https://github.com/antoineforterax-cmd/ARETOVUS/actions/runs/41462436211). | Noyau expérimental testé, pas audité. |
| C1, admission hostile locale | [#165](https://github.com/antoineforterax-cmd/ARETOVUS/pull/165) intégrée : [run #41465517000](https://github.com/antoineforterax-cmd/ARETOVUS/actions/runs/41465517000) : 32 sessions lentes, wallet de contrôle, RAM/CPU du nœud, récupération ; [CI de main #41466261226](https://github.com/antoineforterax-cmd/ARETOVUS/actions/runs/41466261226) réussie. | Quota NAT réel, WAN, hôtes multiples, attaques distribuées non vérifiés : issue #4 ouverte. |
| C2, reorg/partitions | [#166](https://github.com/antoineforterax-cmd/ARETOVUS/pull/166) intégrée après deux runs du candidat [#41466377005](https://github.com/antoineforterax-cmd/ARETOVUS/actions/runs/41466377005) et [#41466418051](https://github.com/antoineforterax-cmd/ARETOVUS/actions/runs/41466418051) réussis. Commit intégré `fdfe5553ca7b005bbea0342b537b73158b2301a7`. | [CI `main@fdfe5553` #41467360190](https://github.com/antoineforterax-cmd/ARETOVUS/actions/runs/41467360190) `completed/success` ; partitions sur un seul hôte, pas deux PC physiques. |
| C3, TCP 100 001+ | [#167](https://github.com/antoineforterax-cmd/ARETOVUS/pull/167) **fusionnée** en `main@44e81503ef502fc288defd34a8e8e87a1e8efb61`; [test natif #41466739781](https://github.com/antoineforterax-cmd/ARETOVUS/actions/runs/41466739781) **completed/success**, artefact `c3-native-tcp-100001-evidence` présent. Ancienne preuve [#116](https://github.com/antoineforterax-cmd/ARETOVUS/pull/116) non substituable à celle de la branche native. | **Palier 100 001 transféré via TCP loopback sur branche native**, mais RAM publiée = processus `cargo test`, non RSS nœud isolé ; aucune preuve de réseau physique à deux hôtes ni validation des performances après fusion de tous les changements. |
| C4, endurance | Courtes validations CI réussies, campagnes physiques antérieures 4 h inconclusives ; aucune preuve complète et récente de 24 h ou 72 h. | **Bloquant** avant extension importante. |
| D1 et D2 | Profils et simulations préexistants, sans preuve de performances physiques sur 1 M–500 M de vrais blocs. | Ne pas confondre simulation de hauteur et blocs effectivement propagés. |
| D3 et D4 | Packaging Win/Linux/macOS existant, mais assistants LAN #127/#128 et documentation historique contenant AURORA non réconciliés. | Nettoyage/qualification encore nécessaires. |
| E1 | **Différée volontairement**, aucune revue humaine indépendante confirmée. | **Bloquant pour tout mainnet ou discours de sécurité sur fonds réels.** |
| E2 | `.github/release-request.json` : `ready=false`, `source=null`, `tag=null`. [#168](https://github.com/antoineforterax-cmd/ARETOVUS/pull/168) **fusionnée** dans `main@059f2f07498ecd457edfdd6964c108d8136b8c64` ; [CI des ZIP #41467090135](https://github.com/antoineforterax-cmd/ARETOVUS/actions/runs/41467090135) et [packages trois OS #41467090198](https://github.com/antoineforterax-cmd/ARETOVUS/actions/runs/41467090198) `completed/success` sur SHA candidat. Packaging au contenu identique et métadonnées déterministes ; **pas de preuve de compilation binaire reproductible**. CI postfusion de `main` encore active au pointage ; binaires non signés. | Aucune nouvelle release autorisée ; reproductibilité complète de compilation non démontrée. |

## Conditions de réévaluation

1. Clore les déficits techniques C3/C4 avec preuve de récupération, ressources du **nœud**, log, SHA, version, temps et topologie, dont un essai authentique sur machines distinctes si exigé ; ne pas simuler 72 h.
2. Terminer D1–D4 et valider les trois systèmes sur des archives issues du même commit épinglé, avec les limites de compatibilité documentées.
3. Finir les contrôles E2 (métadonnées ZIP, vérification des checksums hors archive, provenance, sauvegarde/restauration après mise à jour, signatures si requises), sans supposer que des binaires différents par OS sont identiques.
4. Préserver l'identité native ARETOVUS et les anciens journaux comme preuves historiques ; ne jamais réinitialiser les soldes ou migrer silencieusement les états d'anciens réseaux.
5. Avant **tout** passage mainnet ou fonds réels, réouvrir E1 et obtenir une revue qualifiée indépendante. Un avis d'agent ou un CI vert ne la remplace pas.

## Porte de publication

Le fichier `.github/release-request.json` doit **rester `ready=false`**. Le statut du présent document est une décision anticipée **NO-GO**, pas l'exécution finale de toutes les étapes de la feuille de route. Mettre à jour les faits et les URLs de runs avant toute future décision, en réévaluant aussi les échecs et risques résiduels.
