# ARETOVUS — Instructions de développement et d'intégration pour Codex

_Mise à jour : 29 octobre 2026. À utiliser avec [AGENTS.md](https://github.com/antoineforterax-cmd/ARETOVUS/blob/feature/emission-schedule-v0/AGENTS.md) lorsqu'il est présent sur la branche de travail._

## 1. Mission et autorité

ARETOVUS est un testnet expérimental en Rust. Le responsable du projet décide de l'architecture, des orientations, des priorités et des règles économiques. Codex aide à implémenter, vérifier, documenter et repérer les défauts. Les propositions d'un agent, les tests réussis et l'ouverture d'une PR ne valent **pas** approbation d'un changement de consensus.

**Interdictions sans validation explicite du responsable :** modifier la genèse, l'offre, l'émission, les frais, les signatures admises, l'encodage consensus, le modèle de comptes, le choix de chaîne, les horodatages validés, les règles de réorganisation ou le format réseau incompatible ; fusionner vers `main` ; présenter le testnet comme sûr pour de vrais fonds. Ne pas introduire de porte dérobée, de récupération forcée ni de dépendance critique non examinée.

Ne jamais employer de véritables fonds ni de clés privées réelles pour les tests. Respecter `SECURITY.md` pour les vulnérabilités.

## 2. État des branches à vérifier en premier

Au **29 octobre 2026**, `main` contient un prototype historique. La PR #1 propose `docs/phase-0-protocol` vers `main`. La PR #2 propose `feature/emission-schedule-v0` vers `docs/phase-0-protocol`. Les PR #12 à #36 ouvertes au moment du relevé sont des brouillons ; il faut reconsulter GitHub à chaque reprise.

### Branches indépendantes issues du testnet

- #12 documentation stratégique ; #13 tests d'auto-transfert ; #14 simulation de difficulté ; #15 réconciliation de documentation ; #16 admission P2P.
- #17 benchmark du mempool, puis #18 cache de signatures.
- #19 fuzzing des états avec cache, issue de #18, est **parallèle** à #20. Ne pas la confondre avec un maillon linéaire de #20.

### Chaîne A : tests, réseau et mempool

`feature/emission-schedule-v0` → #17 → #18 → #20 → #21 → #22 → #23 → #24.

### Chaîne B : intégration et portefeuille

`feature/emission-schedule-v0` → #25 → #26 → #27 → #28 → #29 → #35 → #36.

**Ne présumer aucune fusion.** Vérifier pour chaque PR le numéro, la base, la tête, le SHA, le statut draft, les fichiers modifiés, les conflits et les workflows correspondants. Si les branches ou les bases ont changé depuis cette fiche, recalculer le graphe avant toute opération.

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

## 6. Ordre de consolidation conseillé

1. **Fondations :** vérifier #1 puis #2 et leurs différences par rapport à `main`. Préparer une intégration testée et versionnée ; ne pas déclencher une fusion générale automatiquement.
2. **PR indépendantes :** examiner #12 à #16 séparément ; régler les conflits et revalider les branches avec la base à jour.
3. **Chaîne A :** #17, #18, puis examiner la bifurcation #19 / #20 avant d'intégrer #21, #22, #23 et #24. Utiliser `merge`, `rebase` ou `cherry-pick` seulement après avoir expliqué leurs conséquences sur l'historique et les tests. Éviter les doublons de commits.
4. **Chaîne B :** intégrer et tester #25, #26, #27, #28, #29, #35, #36 dans l'ordre de leurs bases, en portant une attention renforcée à la récupération des fonds et aux privilèges de signature.
5. **Version commune :** rejouer CI, tests multi-processus, invariants économiques, simulation des réorganisations et tests P2P/wallet sur la combinaison finale. Vérifier spécialement les interactions entre les chaînes A et B.
6. **Validation sur deux PC :** deux machines réellement distinctes, versions/profils compatibles, dossiers indépendants, synchro bidirectionnelle, transactions signées, redémarrage et partition/reconnexion. Archiver hauteur, hash, latences, logs, caractéristiques des hôtes et résultats.
7. **Limites P0 et endurance :** finir les issues #3/#4/#5, synchro réelle 100k+, campagne 72 h avec rapport final, pannes réelles, fuzzing prolongé et examen extérieur qualifié avant toute revendication mainnet.
8. **Évolutions futures :** confidentialité native, anti-arnaque et récupération avancée, gouvernance de trésorerie, interface graphique et mobile seulement avec architecture, décision et programme de tests distincts.

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
