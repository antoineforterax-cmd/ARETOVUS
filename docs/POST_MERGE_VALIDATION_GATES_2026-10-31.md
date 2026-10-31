# AURORA — état vérifiable du testnet et portes de validation

**Pointage du 31 octobre 2026.** Ce document décrit des preuves vérifiées à des SHA précis, **pas une certification de sécurité**. Les commits et conclusions ci-dessous sont historiques : vérifier les workflows au SHA de `main` avant de les réutiliser. Aucun fonds réel.

## Changements intégrés et preuves

| Chantier | PR / SHA de fusion | Preuve disponible | Limite |
| --- | --- | --- | --- |
| Archives Windows, wallet/SPV sur deux nœuds extraits | [#79](https://github.com/antoineforterax-cmd/AURORA/pull/79), `ffce28b4d9b20704ddeb2d6778a46dd4dee480a6` | [Archives CI réussies](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41141332457) | Deux processus sur runner, pas deux PC physiques |
| Reprise après saturation P2P par préfaces lentes / trickle | [#82](https://github.com/antoineforterax-cmd/AURORA/pull/82), `101447d6de732f245a7de84d6e5d073079758958` | [P2P admission réussie](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41141609643) | Pas d'attaque publique multi-hôtes |
| Profilage 2 048 et 8 192 blocs | [#83](https://github.com/antoineforterax-cmd/AURORA/pull/83), `7633c16f66ca6942da8dc91012ebac5edf759af2` | [CI intégrée réussie](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41141695407) | Stockage Regtest synthétique |
| Confirmation intégrale et inspection hors ligne | [#95](https://github.com/antoineforterax-cmd/AURORA/pull/95), `408ba4c209545584f403042ac42d3d1df599d3c8` | [CI intégrée réussie](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41194597960) | Pas de signataire matériel indépendant |
| Vérification indépendante des champs signés | [#96](https://github.com/antoineforterax-cmd/AURORA/pull/96), `e8704d61891fb8b71ff383e5e8a2e6f16b56601e` | [CI intégrée réussie](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41194871301) | La référence de paiement doit provenir d'une source fiable |
| Contrôle facultatif du TXID avant diffusion | [#97](https://github.com/antoineforterax-cmd/AURORA/pull/97), `660410194c255bb62caf25f34226fbbc342da7bc` | [CI intégrée réussie](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41195139926) | Sans `--expected-txid`, le contrôle facultatif n'est pas actif |
| Ouverture anti-symlink des fichiers sensibles | [#98](https://github.com/antoineforterax-cmd/AURORA/pull/98), `881ef799d6a705e49ba9a629c4f44bd090ebc79e` | [CI intégrée réussie](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41195404105) ; [tests Linux/Windows/macOS à la tête de PR](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41194150456) | Substitution de dossiers parents et machine compromise non couvertes |
| Véritable ENOSPC sur disque temporaire isolé | [#88](https://github.com/antoineforterax-cmd/AURORA/pull/88), `8c3133f505df79997c0011c109bc9b02b9ad2f47` | [ENOSPC tmpfs à la tête de PR réussi](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41143685735) ; [CI intégrée réussie](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41195723377) | Ne simule pas une coupure électrique |
| Transfert de sauvegarde v6 entre deux runners, restauration, réorganisation | [#89](https://github.com/antoineforterax-cmd/AURORA/pull/89), `d2b7466db44859dd64ce573c6c3132ac9d9206bb` | [Workflow cross-runner réussi à la tête de PR](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41143881862) | Runner CI n'est pas un PC de terrain ; vérifier séparément la CI du commit de fusion |

La fusion d'une PR ne vaut pas validation du nouveau commit combiné ; l'ensemble `main` doit être vérifié après chaque intégration.

## Mesures sur branches de test, distinctes des fusions

- [PR #87 — un million de blocs en persistance](https://github.com/antoineforterax-cmd/AURORA/pull/87) : [workflow réussi au SHA `b97a30a`](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41143906083). Exactement **1 000 001 blocs connus** (genèse comprise), construction 70,0 s, sauvegarde 1 329,6 s, restauration 72,9 s ; 637 336 KiB de RSS maximale ; 1 849,86 s pour le test complet. Ce n'est **pas** la synchronisation P2P d'un million de blocs.
- [PR #85 — matrice de régression massive](https://github.com/antoineforterax-cmd/AURORA/pull/85) : [cinq jobs réussis au SHA `3db314f`](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41143191869) : 100 003 blocs connus en stockage, **trois** tours de partitions, scénarios P2P hostiles, 8 192 blocs profilés, **72 cycles de soak local durant ~16 minutes**. Ce n'est **pas** une campagne de 72 heures.
- [PR #86 — contrôle renforcé des preuves de profilage](https://github.com/antoineforterax-cmd/AURORA/pull/86) : [workflow storage-phase-profile réussi](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41143170383).

Ces preuves de branches ne doivent pas être attribuées à `main` tant que les PR correspondantes ne sont pas fusionnées et la CI intégrée passée.

## Portes de sortie vers un testnet public durable

1. CI, build verrouillé, Rustfmt, Clippy, tests Rust, smoke multi-nœuds et archives multiplateformes réussis sur **le SHA intégré exact**.
2. Synchronisation réseau en conditions réelles au-delà de 100 000 blocs avec forks, partitions, reprise et mesure disque/RAM/CPU.
3. Deux **PC physiques** indépendants (NAT, dérive d'horloge, coupures réseau, transfert hors hôte) avec comparaison de la hauteur **et du hash du sommet**.
4. Soak **72 heures réelles**, bilan conservé et signé par les opérateurs, pas 72 cycles.
5. Panne électrique/disque matériel, sauvegardes dégradées et restauration après corruption, avec critères de perte documentés.
6. Relecture humaine indépendante du consensus, de la cryptographie, du wallet et du stockage avant toute affirmation de sécurité mainnet.

**Le testnet reste expérimental.** Ne pas déployer comme mainnet sécurisé ni recommander l'usage de fonds réels ; ne pas confondre résultats automatisés et audit indépendant.
