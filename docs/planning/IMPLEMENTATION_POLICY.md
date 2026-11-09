# ARETOVUS — Instructions de développement et d'intégration pour Codex

_Mise à jour opérationnelle : 2 novembre 2026 ; contexte historique du 29 octobre conservé dans Git. Consulter [AGENTS.md](../../AGENTS.md) sur le **même SHA** que le code et [la feuille de route V2 #178](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178) et [l'historique #142](https://github.com/antoineforterax-cmd/ARETOVUS/issues/142). Le dépôt est un réseau expérimental natif ARETOVUS, non le prototype des branches initiales._

## 1. Mission et autorité

ARETOVUS est un testnet expérimental en Rust. Le responsable du projet décide de l'architecture, des orientations, des priorités et des règles économiques. Codex aide à implémenter, vérifier, documenter et repérer les défauts. Les propositions d'un agent, les tests réussis et l'ouverture d'une PR ne valent **pas** approbation d'un changement de consensus.

**Interdictions sans validation explicite du responsable :** modifier la genèse, l'offre, l'émission, les frais, les signatures admises, l'encodage consensus, le modèle de comptes, le choix de chaîne, les horodatages validés, les règles de réorganisation ou le format réseau incompatible ; fusionner vers `main` ; présenter le testnet comme sûr pour de vrais fonds. Ne pas introduire de porte dérobée, de récupération forcée ni de dépendance critique non examinée.

Ne jamais employer de véritables fonds ni de clés privées réelles pour les tests. Respecter `SECURITY.md` pour les vulnérabilités.

## 2. État de référence, dépendances et archives

Le réseau natif ARETOVUS a été intégré sur `main` le 1er novembre 2026. **Relever à chaque session** le SHA courant, la topologie des PR, leurs dépendances et les contrôles au SHA exact ; les relevés du 29 octobre et les anciennes bases `feature/emission-schedule-v0` sont des **archives**, pas la vérité opérationnelle.

- A1–A4, B1–B4, C1 : intégrations et limitations consignées dans l'[issue #142](https://github.com/antoineforterax-cmd/ARETOVUS/issues/142).
- C2 : [PR #166](https://github.com/antoineforterax-cmd/ARETOVUS/pull/166) intégrée (tests réorganisations et partitions locaux) ; C3 : [PR #167](https://github.com/antoineforterax-cmd/ARETOVUS/pull/167) intégrée (100 001 blocs TCP loopback, pas réseau physique).
- E2 : [PR #168](https://github.com/antoineforterax-cmd/ARETOVUS/pull/168) améliore la reproductibilité **des métadonnées ZIP**, pas celle des exécutables ; E3 : [PR #169](https://github.com/antoineforterax-cmd/ARETOVUS/pull/169) documente un **NO-GO**, pas un lancement.
- Les PR historiques #108/#109/#112/#115/#116/#124–#131 incluent des conceptions, des branches empilées, des précurseurs et des résultats parfois anciens. Conserver leur provenance, revoir leurs différences avant toute reprise ; ne pas fusionner directement les assistants LAN sous l'ancienne marque.
- Le responsable a placé **E1, revue par une personne qualifiée extérieure au projet, en ultime étape finale**, sans l'annuler et sans prétendre qu'il est accompli. Ses propres revues du code et celles des agents constituent des vérifications internes, pas cette étape extérieure. Aucun mainnet ou fonds réel avant preuves et décisions adaptées.

## 3. Ordre obligatoire pour chaque session de travail

1. **Inventorier.** Lire `README.md`, `CONTRIBUTING.md`, `docs/CONSENSUS.md`, `docs/TESTNET_DECISIONS.md`, `docs/FINALIZATION_CHECKLIST.md`, `docs/TESTNET_VALIDATION_REPORT.md`, les issues critiques et les PR ouvertes. Recenser les commits et workflows réellement disponibles. Identifier les documents historiques dépassés.
2. **Cartographier.** Construire un tableau `PR / base / tête / SHA / dépendances / CI / conflit / statut / action proposée`. Repérer les branches parallèles, les modifications communes et les écarts de provenance. Ne pas compter une PR empilée comme fusionnée parce qu'une autre a passé la CI.
3. **Choisir un chantier limité.** En priorité : correction ou intégration vérifiable de l'existant. Définir le comportement attendu, les invariants, les risques, les critères d'acceptation et les fichiers concernés. Pas de refonte au passage.
4. **Reproduire avant de corriger.** Ajouter un test de régression qui échoue sur le comportement défectueux, lorsqu'un défaut est reproductible. Épingler les commits comparés et conserver les journaux utiles.
5. **Implémenter sur une branche isolée.** Petits changements, commits cohérents, dépendances vérifiées, pas de secrets. Ne pas réécrire plusieurs branches empilées en parallèle ; éviter les conflits de fichiers et les mises à jour de base non planifiées.
6. **Tester en couches.** Lancer les contrôles du §4, puis les tests de sécurité ciblés. Pour les risques de protocole, vérifier aussi l'absence de dérive des règles de consensus.
7. **Organiser une contre-revue adversariale.** Demander si possible à un sous-agent distinct, sans lui fournir uniquement l'explication de l'auteur, de chercher des contre-exemples et d'écrire des scénarios reproductibles selon le §5. Les sous-agents ne doivent pas travailler simultanément sur les mêmes fichiers.
8. **Corriger, puis recommencer les tests.** Aucun résultat positif d'avant la correction ne valide le nouveau SHA. Si le défaut n'est pas résolu, laisser la PR en brouillon et documenter précisément le blocage.
9. **Préparer la PR.** Mentionner commit, base, dépendances, risques, tests exécutés, tests non exécutés, mesures avant/après et compatibilité. Mettre à jour les documents concernés sans effacer l'historique.
10. **Stopper avant fusion.** Proposer l'ordre d'intégration au responsable. Ne pas fusionner ou supprimer de branches sans instruction explicite. Après toute fusion autorisée : revalider l'ensemble sur le nouveau commit, puis seulement traiter le maillon dépendant suivant.

## 4. Batterie de tests et critères de succès

Exécuter depuis la racine du projet, dans un environnement adapté :

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --locked
cargo test --workspace --locked
python3 scripts/testnet_smoke.py
```

Ajouter selon le code concerné : `scripts/testnet_soak.py`, campagne libFuzzer, tests d'intégration TCP et wallet, tests NIST ML-DSA, bench mempool, scripts de retarget, partitions, essais de reprise et tests des workflows GitHub. Contrôler l'existence et les arguments exacts des scripts avant de lancer les commandes. Éviter de lancer en parallèle des scénarios qui utilisent le même répertoire de données ou les mêmes ports.

**Invariants à contrôler :** offre totale et émission, conservation des montants, débordements, nonces et relecture des transactions, absence de mutation partielle après rejet, signatures non canoniques, forks et réorganisations, priorité du travail cumulé, frais/burn, état du mempool et compatibilité des formats. Pour toute mesure de performance : conserver machine, commit, durée, taille de l'échantillon et méthode ; ne pas confondre benchmark cryptographique isolé et débit P2P de bout en bout.

Un test est **réussi** uniquement si un journal/une CI correspondant au **SHA exact** le confirme. Distinguer `écrit`, `exécuté`, `réussi`, `échoué`, `non exécuté`, `à confirmer`. Ne jamais annoncer un audit indépendant simplement parce que la CI est verte.

## 5. Sous-agents : tenter de casser le prototype sans mettre d'utilisateurs en danger

Si l'environnement ne permet pas de lancer des sous-agents, réaliser une revue séparée sur un nouveau contexte et déclarer cette limite. Leur rôle est d'attaquer **uniquement des instances locales ou des environnements autorisés**, jamais des hôtes tiers. Fournir à chacun le SHA cible et demander : `scénario / préconditions / fichiers-lignes / impact / reproduction / preuve / correction proposée / test de non-régression`.

| Relecteur | Mission adversariale | Exemples de scénarios |
| --- | --- | --- |
| A — Consensus et économie | Chercher une création de monnaie indue, une double dépense ou un désaccord entre nœuds | Overflow, nonce répété, transfert à soi-même, réorganisations, horodatages futurs, retarget |
| B — Réseau et synchronisation | Tester les entrées hostiles et l'épuisement des ressources | Handshakes lents, saturation après handshake, paquets tronqués, inventaires contradictoires, NAT, partitions, synchronisation longue |
| C — Wallet et stockage | Vérifier les clés, les sauvegardes et l'absence de pertes silencieuses | Mauvais mot de passe, récupération après perte, fichier tronqué, disque plein, crash entre sauvegarde et publication, RPC à trame lente |
| D — Intégration et chaîne d'approvisionnement | Contrôler la cohérence d'une branche intégrée | Dépendances de PR, versionnement, validation de données, dépendances Rust, corpus fuzz, différences de paramètres |

Les relecteurs rendent leurs résultats indépendamment. Les corrections vont sur des branches isolées après triage. Ne jamais demander à un agent de « trouver obligatoirement une faille » : un constat sans reproduction est une hypothèse, et un test sans crash n'est pas une preuve d'absence de faille.

## 6. Ordre opérationnel de consolidation (après migration native)

1. **D4 — traçabilité et règles d'agents.** Inventorier les PR historiques ; intégrer sans perte de créditeurs les corrections de marque et de documentation ; maintenir `AGENTS.md`, cette politique et les guides d'essai cohérents.
2. **D3 — outils multi-OS natifs.** Reconstruire sur `main` les parties sûres et utiles des assistants #127/#128 ; vérifier l'archive et les chemins, modes et binaires ARETOVUS sur Windows/Linux/macOS, sans démarrer de réseau non autorisé ni désactiver les pare-feu.
3. **D1 — profils et optimisation durable.** Mesurer sur SHA épinglé les lectures de snapshots, les contrôles de métadonnées, CPU/RSS du processus nœud distinct, la latence de vérification ; ne jamais supprimer les vérifications de fichiers, le rejeu ou la restauration pour gagner du temps.
4. **Compléter C2/C3/C4, D2 et E2 selon leurs preuves manquantes.** Refaire les expériences longues sur machines distinctes lorsque requises ; des smokes courts ou des modèles de 500 M blocs ne remplacent pas de vrais essais.
5. **E3 — maintenir NO-GO tant que les critères de testnet public sont ouverts**. La préparation d'une checklist n'est pas une décision d'ouverture.
6. **E1 — revue par une personne extérieure au projet en étape ultime** selon la priorité demandée par le responsable. La repousser n'autorise aucune déclaration de sécurité pour des fonds réels.

Chaque fusion demeure conditionnée aux autorisations, aux contrôles au SHA exact et à une contre-revue de sécurité. Les travaux V2 sont suivis dans l'issue #178 et les jalons antérieurs dans l'issue #142 ; vérifier chaque preuve par rapport à son SHA.

## 7. Modèle de compte rendu obligatoire

À la fin d'un chantier, livrer ce récapitulatif :

```text
Objectif et périmètre :
Branche de départ / commit :
Branche de travail / commit final :
PR et dépendances :
Fichiers modifiés :
Comportement avant / après :
Invariants vérifiés :
Tests exécutés (avec liens et SHA) :
Tests non exécutés et pourquoi :
Sous-revues adversariales (preuves, non-reproductions) :
Risques et régressions possibles :
État : terminé sur branche / en revue / prêt à proposer / bloqué / fusionné
Action demandée au responsable :
Prochaine tâche autorisée :
```

Ne pas confondre `terminé sur une branche`, `fusionné dans la branche de référence` et `disponible dans une release`. Le responsable d'ARETOVUS reste décisionnaire pour les changements du protocole et les fusions sensibles.
