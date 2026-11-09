# Documentation ARETOVUS — choisir son parcours

**Un seul sommaire.** Cette page guide les nouveaux lecteurs vers les dossiers utiles, sans mélanger les règles du testnet, les projets futurs et les rapports historiques.

> **Statut : laboratoire blockchain expérimental, aucun mainnet ni fonds réels.** Aucune revue humaine extérieure complète et documentée n'est confirmée ; les vérifications IA et la CI ne sont pas des audits indépendants.

## Où commencer ?

| Je veux… | Ouvrir |
| --- | --- |
| Découvrir la blockchain sans coder | **[01 — Découvrir ARETOVUS](01-demarrer/README.md)** |
| Participer, proposer une idée ou développer avec l'IA | **[02 — Contribuer](02-participer/README.md)** |
| Comprendre les règles, la cryptographie ou les recherches futures | **[03 — Protocole et recherche](03-recherche/README.md)** |
| Installer un testnet, utiliser le portefeuille ou miner des blocs de test | **[04 — Installer et exploiter](04-exploiter/README.md)** |
| Suivre l'avancement et comprendre les décisions contradictoires | **[05 — Pilotage et décisions](05-pilotage/README.md)** |
| Vérifier la sécurité, examiner les risques, proposer un audit | **[06 — Sécurité](06-securite/README.md)** |
| Chercher une preuve, un bug, un benchmark ou un résultat de test | **[07 — Rapports](07-rapports/README.md)** |
| Retrouver un ancien document retiré ou déplacé | **[90 — Archives](90-archives/README.md)** |

## Références techniques à chemins stables

Les documents suivants restent **à la racine de `docs/`** car de nombreux guides, scripts ou procédures les utilisent déjà. Ce n'est **pas un second sommaire**.

- [Protocole](PROTOCOL.md) et [consensus](CONSENSUS.md) : profils expérimentaux, à confronter au code sur le SHA visé.
- [Émission](EMISSION_MODEL.md), [décisions testnet](TESTNET_DECISIONS.md), [registre de phase 0](DECISION_REGISTER.md).
- [Guide technique testnet](TESTNET_GUIDE.md), [exploitation](TESTNET_OPERATIONS.md), [rapport de validation](TESTNET_VALIDATION_REPORT.md).
- [Fuzzing](FUZZING.md), [modèle de menaces](THREAT_MODEL.md).
- [Checklist antérieure](FINALIZATION_CHECKLIST.md) : **archive historique**, pas feuille de route V2.

Les autres sous-dossiers existants (`guide-fr/`, `planning/`, `validation/`, `consolidation/`) regroupent les travaux datés et les procédures avancées. Les déplacer aveuglément casserait les scripts ou les preuves qui les citent ; ils restent accessibles depuis les sections correspondantes.

## Pour les contributeurs

- [Règles de contribution](../CONTRIBUTING.md), [garde-fous des agents](../AGENTS.md), [politique de développement](planning/IMPLEMENTATION_POLICY.md).
- [Licence officielle MIT OR Apache-2.0](../LICENSE), [explication des droits](02-participer/LICENCES_ET_DROITS.md), [provenance IA](../PROVENANCE.md).
- [Feuille de route V2 #178](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178) ; [anciens jalons #142](https://github.com/antoineforterax-cmd/ARETOVUS/issues/142).

**Règle de lecture :** « existe dans un testnet » ≠ « audité pour de l'argent réel » ; « proposé » ≠ « implémenté » ; « une CI est verte » ≠ « un audit indépendant a été effectué ».

[Retour à la page d'accueil](../README.md).
