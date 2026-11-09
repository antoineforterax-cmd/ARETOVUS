# Règles d'organisation et d'entretien de la documentation

**Objet :** empêcher le retour d'un dépôt où les propositions, historiques de tests et règles exécutées se contredisent visuellement. Cette politique concerne la documentation ; elle **ne modifie aucune règle de consensus**.

## Les seules portes d'entrée publiques

- **[README racine](../../README.md)** : présentation courte du projet, sécurité, chemins par profil.
- **[docs/README.md](../README.md)** : sommaire unique de la documentation.
- **[ETAT_ACTUEL.md](../01-demarrer/ETAT_ACTUEL.md)** : état du testnet, fonctionnalités livrées et limites, avec une date et un SHA.
- **[DECISIONS_CLARIFIEES.md](DECISIONS_CLARIFIEES.md)** : divergences entre vieilles propositions et règles documentées.
- **[ARCHIVES_ET_RAPPORTS.md](../ARCHIVES_ET_RAPPORTS.md)** : catalogue des fichiers historiques, sans les faire passer pour des choix actifs.
- **[Issue V2 #178](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178)** : suivi opérationnel V2, pas un substitut au code ni aux résultats CI.
- **[Issue #142](https://github.com/antoineforterax-cmd/ARETOVUS/issues/142)** : historique des jalons précédents.

**Ne pas créer une nouvelle feuille de route principale dans un Markdown** sans expliquer pourquoi les issues de référence ne conviennent plus et sans mettre à jour les points d'entrée.

## Où ranger un nouveau document ?

| Sujet | Emplacement de référence |
| --- | --- |
| Tutoriel progressif pour débutant | `docs/guide-fr/` et sommaire public |
| Spécification de comportement testnet | `docs/PROTOCOL.md`, `CONSENSUS.md`, ou référence explicitement liée |
| Proposition future / ADR | `docs/planning/` ; mention **PROPOSITION** et impacts |
| Preuve de test datée | Rapport de validation ou document dédié avec **SHA, commande, environnement et résultat** |
| Vulnérabilité exploitable | **Canal privé défini dans SECURITY.md**, jamais dans une issue publique avec reproduction sensible |
| Rapport historique | Conserver daté ; référencer dans `ARCHIVES_ET_RAPPORTS.md` |
| Licence, tiers et attribution | `LICENSE*`, `LICENCES_ET_DROITS.md`, `PROVENANCE.md` |

## Chaque document doit répondre à quatre questions

1. **De quoi parle-t-il ?** Explication, procédure, spécification, proposition, rapport de test ou archive.
2. **À quelle version s'applique-t-il ?** Date, branche et **SHA** pour toute preuve de comportement.
3. **Quelle est sa force ?** Règle exécutée du testnet, proposition non approuvée, résultat observé, hypothèse, ou ancienne décision.
4. **Où aller ensuite ?** Lien de retour à l'[index](../README.md) et à la source technique appropriée.

La présence d'un mot « validé » sans numéro de PR, lien vers CI/compte rendu et SHA ne suffit pas. Une CI verte ne prouve pas un audit de sécurité ; une simulation de 500 millions de blocs d'émission ne prouve pas un réseau traitant réellement 500 millions de blocs.

## Quand deux fichiers se contredisent

- **Ne pas effacer le plus ancien automatiquement** : identifier version, décision, test et historique.
- Comparer le **code au SHA visé** et le document de décision **explicitement approuvé**, s'il existe.
- Pour une divergence qui touche le consensus, la cryptographie, les clés, les formats, l'émission ou la récupération : ouvrir une issue/RFC, indiquer l'invariant et **ne pas réécrire la règle pour « faire correspondre les docs »**.
- Si une vieille proposition n'a jamais été activée, la classer comme **historique / non implémentée**, et lier [DECISIONS_CLARIFIEES.md](DECISIONS_CLARIFIEES.md).
- Ne jamais faire disparaître un résultat négatif de test ni supprimer l'origine d'un ancien choix.

## Avant de proposer un tri ou une suppression de fichiers

- Vérifier les **liens entrants** depuis guides, scripts, workflows, PR et issues ; une référence externe peut subsister.
- Préférer **marquer « archive » et retirer du parcours débutant** à déplacer/supprimer un rapport de sécurité ou de test.
- Pour un déplacement légitime, corriger les liens relatifs et fournir si nécessaire une page de renvoi à l'ancien chemin.
- Ne pas modifier le contenu scientifique d'un rapport historique pour le faire sembler valable pour une version nouvelle.
- Vérifier les liens **après** modifications, puis vérifier que seuls les fichiers prévus figurent dans la PR.

## Contrôle de livraison documentaire

- [ ] Sommaire et README reliés à la nouvelle page.
- [ ] Aucun exemple présenté comme compatible avec toutes les versions sans preuve.
- [ ] Tous les liens Markdown relatifs vérifiés contre l'arbre Git de la branche.
- [ ] Aucun changement de code, consensus ou licence juridique glissé dans la PR de documentation.
- [ ] Statut fonds réels / audit humain extérieur honnête.
- [ ] Changements récapitulés dans une PR isolée et revue, sans fusion automatique.

**Objectif :** une documentation courte en surface, profonde sur demande, traçable et ouverte aux nouveaux venus — sans faire passer l'organisation des fichiers pour une validation technique du protocole.
