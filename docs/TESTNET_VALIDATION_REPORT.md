# Rapport d’essais du testnet AURORA

## Relevé des vérifications au 29 octobre 2026

Ce rapport garde les résultats de chaque campagne avec leur date et leur commit. Depuis sa dernière mise à jour, plusieurs PR supplémentaires ont publié des résultats de CI :

| PR | Vérification annoncée | Limite |
| --- | --- | --- |
| [#13](https://github.com/antoineforterax-cmd/AURORA/pull/13) | Rejet atomique des auto-transferts ML-DSA-65, tests Rust et smoke réussis | Branche non fusionnée |
| [#14](https://github.com/antoineforterax-cmd/AURORA/pull/14) | Simulateur indépendant du retarget, 323 vecteurs et sept traces de 30 blocs ; CI réussie | Pas d'essai réseau avec horloges divergentes |
| [#16](https://github.com/antoineforterax-cmd/AURORA/pull/16) | Mesures de préfaces P2P lentes, tests de synchronisation et de reprise ; CI réussie | Attaques après handshake et NAT encore à tester |
| [#17](https://github.com/antoineforterax-cmd/AURORA/pull/17) et [#18](https://github.com/antoineforterax-cmd/AURORA/pull/18) | Benchmark de signatures et cache borné, avec CI réussie | Gain cryptographique ciblé, pas débit global du réseau |
| [#19](https://github.com/antoineforterax-cmd/AURORA/pull/19) | 339 455 exécutions de fuzzing en 301 s, sans divergence rapportée ; rejeu du corpus | Campagne courte, profil Regtest Ed25519 |
| [#20](https://github.com/antoineforterax-cmd/AURORA/pull/20) et [#21](https://github.com/antoineforterax-cmd/AURORA/pull/21) | Tests de réorganisation avec cache et réinsertion des paiements orphelins ; CI réussie | Mempool non persistant ; PR non fusionnées |
| [#22](https://github.com/antoineforterax-cmd/AURORA/pull/22) | Parseur hexadécimal strict ; CI du dernier commit réussie | Revue et intégration à terminer |

Ces résultats proviennent des descriptions des PR et des workflows associés. Ils n'ont pas été reproduits indépendamment dans cette mise à jour. Toutes ces PR restent en brouillon. Leur réussite séparée ne prouve pas que leur intégration commune passera.

La campagne de 72 h démarrée le 28 octobre à 13:23:40 UTC était encore notée « en cours » dans le rapport précédent. **Son résultat final n'est pas confirmé** ; il ne faut pas la compter parmi les essais réussis. Le test de 100 003 blocs concerne le stockage et la restauration, pas la synchronisation P2P du même volume.

---


## Campagne Ubuntu en cours — 28 octobre 2026

La [CI de `6e6aaa0`](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40695381103) a réussi, y compris les tests NIST, les nouveaux scénarios TCP, le smoke et le lanceur avec paiements. Les quatre tests NIST ont également réussi localement dans Ubuntu. Le test court Ubuntu a confirmé six blocs, cinq paiements, deux redémarrages et la restauration de sauvegarde en 124,21 s (`short_run_passed`).

La campagne de 72 h a démarré le **28 octobre 2026 à 13:23:40 UTC** dans Ubuntu/WSL2, session tmux `aurora-essai`, avec trois nœuds, un mineur et deux comptes jetables. Le premier bloc a convergé. Il s'agit d'un essai **en cours**, pas d'un succès de 72 h. Les résultats finaux devront confirmer la durée, les paiements, les redémarrages et la restauration. Le lancement Linux n'a modifié aucune protection Windows.

Provenance : binaires compilés depuis `f6afc7f38df50b2f7cd9a027096fedec48be3822`; lanceur et checkout au `6e6aaa03b8dfa4100686e75a39146f1e79a37714`. L'unique différence est la correction Clippy du décodeur hexadécimal dans le test NIST, sans changement du code des binaires. Empreintes SHA-256 : nœud `2935c1227a27c90133ca5915660bad27133ab750f58365efb50c5ca6896e6687`, wallet `b9638a08c9b2c05092999ac979e8c66e899a4f9431b67c793e373f5f55c76d7d`, lanceur `bf116d0c5e7f617e5ebc8b5918f140b080dcc127ee38849440cc098fc04b655e`. Les JSON du test court et de la campagne, les journaux et les commits sont à conserver. Ne pas publier les clés jetables.

## Mise à jour du 28 octobre 2026

La [Release expérimentale.1](https://github.com/antoineforterax-cmd/AURORA/releases/tag/testnet-v0.0.1-experimental.1) publie les binaires de `421d443370e83a9ffa0c128da9f09d6f99b0ef8f` pour Windows x86-64, Linux x86-64 et macOS arm64, avec SHA256SUMS. Le [workflow de publication](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40528021813) a réussi, comme la CI, le soak et le fuzzing de cette source. Ces binaires sont non signés et non audités par un humain.

Le [test de volume](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40527921129) a réussi : hauteur 100 001, 100 003 blocs connus avec genèse et branche secondaire, sauvegarde puis rechargement v6. Construction 7,7 s, sauvegarde 69,9 s, contrôle de restauration à 87,0 s; test complet avec nettoyage 100,72 s. La commande, compilation comprise, a atteint 408 240 Kio de RSS maximal. Ce sont des blocs Regtest vides sans difficulté, pas une preuve de synchronisation P2P de 100 000 blocs ni une charge de nombreux comptes.

Le lanceur avec paiements a passé un essai local distinct sur les binaires `421d443` : 6 blocs, 5 paiements confirmés, 2 redémarrages du pair B, sauvegarde restaurée, 82,49 secondes. Le résultat est `short_run_passed`, jamais un succès de 72 h. Le lancement long sur le Bureau a échoué avant le premier bloc le 28 octobre avec Windows 4551; le porteur a confirmé le refus Device Guard dans son terminal. Aucune protection Windows n'a été modifiée. Une procédure Ubuntu/WSL est préparée; la campagne longue reste à réaliser.

Les sections ci-dessous sont l'historique des campagnes antérieures et gardent leurs propres commits et limites. Les mentions anciennes de volume non réalisé ou d'absence de Release sont dépassées par cette mise à jour. Le libellé historique du wallet `confirmed_balance` désignait en réalité le solde après application du mempool; la branche le corrige en `pending_adjusted_balance`, sans changement RPC.

**Commit de code et CI multi-nœuds :** `37ec33c2ccc3e1d1be9dd6c2085c1a843e581578`  
**Date des workflows :** 26 octobre 2026  
**Branche :** `feature/emission-schedule-v0`  
**Verdict :** les contrôles automatisés ciblés passent; ce résultat ne constitue ni un audit indépendant ni une garantie pour des fonds réels.

## Contrôles de compilation et tests

- [CI du verrou de dossier et du smoke test séparant mineur et serveur](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40402865286) : formatage, Clippy avec avertissements refusés, compilation workspace, tests et smoke test testnet — succès.
- Le correctif de lecture réseau indexe les transactions confirmées, suit les réorganisations et sert les pages de blocs sans trier toute la chaîne sous le verrou; le smoke couvre toujours statut, preuve Merkle et synchronisation. Une mesure sous charge hostile reste à faire.
- [Contrôle RustSec des dépendances](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40402139677) : 78 dépendances verrouillées inspectées, aucune vulnérabilité signalée par la base chargée lors de l’exécution. Ce contrôle ne vérifie pas la logique du protocole.
- [CI de l’ajout du rattrapage automatique](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40333338987) : mêmes contrôles, y compris le scénario d’un pair en fonctionnement qui rejoint les hauteurs 21 puis 22 — succès.
- [Campagne libFuzzer sur le nœud avec verrou de dossier](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40361895495), commit `31b016e931624785508cf82b464efa19557c31bf` : trois cibles exécutées 301 secondes chacune — succès, sans crash rapporté. Le changement suivant adapte seulement le smoke test au verrou et est couvert par la CI courante.

## Scénario multi-nœuds

Le smoke test utilise plusieurs processus isolés et des dossiers temporaires :

1. Lance un serveur puis vérifie qu’un second processus ne peut pas miner dans le même dossier; le mineur actif utilise un dossier distinct. Mine quatre blocs et vérifie l’émission ainsi que la répartition 95 % mineur / 5 % trésorerie.
2. Crée et utilise un wallet testnet, vérifie son entrée au mempool, son inclusion et sa preuve d’inclusion Merkle avec un minimum de confirmations.
3. Synchronise les nœuds, redémarre un serveur depuis son snapshot et fait télécharger le pair frais.
4. Mine jusqu’à la hauteur 20, couvrant deux retargets, pousse cette chaîne, redémarre le nœud receveur et vérifie la synchronisation de 16 blocs.
5. Crée une branche concurrente de hauteur 21 depuis la genèse, vérifie que son travail cumulé provoque la réorganisation du nœud, puis vérifie la convergence d’un autre pair. La transaction devenue orpheline n’est plus rapportée comme confirmée.
6. Lance un pair avec un serveur déjà actif, vérifie qu’il rejoint automatiquement une branche de hauteur 21, puis qu’il rattrape un nouveau bloc de hauteur 22 sans commande `sync` manuelle.
7. Soumet une nouvelle transaction à un pair, vérifie son relais périodique à un second pair, puis la fait miner par un nœud qui ne contacte que le second pair.
8. Un test Rust fait couper un pair au milieu d’une page de blocs et vérifie qu’aucun bloc partiel ne modifie la chaîne ni le disque. Arrête brutalement un mineur après plusieurs index validés, puis vérifie qu’il redémarre au-dessus du dernier index complet.
9. Sur Linux, impose une limite de taille aux écritures pour simuler un échec de sauvegarde; vérifie que le nœud reprend l’index complet précédent. Les tests Rust restaurent aussi une copie complète du dossier v5 et lisent un snapshot v4 avant écriture v5.

Le journal CI résume les assertions : « wallet, SPV, 20-block soak, heavier-branch reorganization, live peer catch-up, mempool relay, forced-stop and write-limit recovery passed ». Résultat : succès sur le commit indiqué.

## Essai prolongé et archives testnet

- [Essai à trois nœuds avec verrou de dossier](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40361895189) : 36 blocs propagés et vérifiés sur trois processus en **490,2 secondes**; le pair intermédiaire a été arrêté puis redémarré quatre fois, et une copie complète de son dossier a été restaurée et vérifiée au même sommet — succès. Cet essai est local au runner CI; il ne prouve pas un fonctionnement continu sur plusieurs hôtes ou pendant des jours.
- [Construction des archives et essai du verrou sur les trois systèmes](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40400631436) : compilation Rust release et empaquetage du nœud et du wallet réussis sur Windows, Linux et macOS. Sur chaque système, un test lance le serveur, vérifie qu’un second écrivain est refusé, arrête le serveur, puis mine et vérifie un bloc avec le dossier libéré. Chaque archive contient les binaires, la documentation et `SHA256SUMS.txt`, vérifié pendant l’empaquetage. Ces artefacts CI non signés expirent après 14 jours; le dépôt est public depuis le 27 octobre 2026, mais aucun installateur ni publication GitHub Release n’existe encore.

## Essai manuel fourni par le porteur

Le 26 octobre 2026, le porteur a exécuté sur Windows une archive testnet construite par CI (révision `237e87d9a33c`). La sortie PowerShell fournie montre : refus d’une confirmation de mot de passe différente, création d’une clé chiffrée `AURKEY4`, minage avec le même dossier de données sur plusieurs invocations jusqu’à la hauteur **367** (vérification locale explicitement réussie à 311; sortie de minage fournie jusqu’à 367), et démarrage du serveur sur `127.0.0.1:18444`. Le wallet a lu un solde confirmé de **1 424,99857500 AUR** sur le serveur lancé à la hauteur 3; un second dossier a synchronisé ces 3 blocs puis passé `testnet verify` (4 blocs connus avec la genèse). Les retargets observés sur les premières invocations suivent 8→6→8→10→12→14 bits. Une seconde tentative de `serve` sur le même port a été refusée par Windows avec l’erreur 10048; le port était déjà occupé. Le serveur doit être arrêté puis relancé pour charger les blocs minés ultérieurement; une nouvelle commande de synchronisation a accepté **0** bloc et le second dossier est resté à 3, car l’ancien serveur occupait toujours le port. La synchronisation jusqu’à 367 n’est pas encore démontrée. L’adresse et le chemin locaux ne sont pas recopiés ici. Le porteur rapporte aussi que les binaires macOS et Linux démarrent sur d’autres PC; aucun journal de synchronisation entre ces machines n’a été fourni. Ce retour manuel ne remplace pas un essai réseau multi-hôtes vérifié.

## Campagnes fuzz

Les trois cibles ont été exécutées pendant 301 secondes chacune sur le commit `31b016e931624785508cf82b464efa19557c31bf` :

- `canonical_decode` : **52 129 549** entrées sur les décodeurs canoniques transaction, compte/bloc et en-tête.
- `merkle_proofs` : **55 443 335** entrées sur le vérificateur de preuves Merkle.
- `network_payloads` : **60 256 094** entrées sur les parseurs de charges P2P, blocs et listes de pairs.

Aucun crash n’a été rapporté pendant ces campagnes. Les journaux de campagne sont dans le workflow lié ci-dessus. Ces essais CI sont bornés; ils ne remplacent pas un fuzzing prolongé avec corpus conservé et revue humaine indépendante.

## État des changements v6 au 27 octobre 2026

La branche a ajouté l’ordre d’arrivée stable pour les pages P2P, un index de stockage v6 paginé et sa lecture avec compatibilité v4/v5. La [CI du commit `21a24c7`](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40485261614) a réussi : formatage, Clippy, compilation, tests Rust et smoke multi-nœuds. Les journaux des tests Rust incluent les cas de migration v5→v6, corruption d’une page, rétention de pages et compte malformé. Les [archives Windows, Linux et macOS](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40485261646) ont aussi été construites avec succès. La [CI de la révision documentaire `21e5fe0`](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40485399202) est également verte.

Le contrôle local `cargo fmt --all -- --check` a réussi avec Rust 1.99.1 sous Windows. Le contrôle local `cargo clippy`/compilation n’a pas abouti : la cible MSVC manque de `link.exe`, et le contrôle d’applications Windows refuse l’exécution des scripts de construction de la cible GNU (erreur 4551). La CI Linux fournit ici la preuve de compilation et de tests. Sur le code P2P u64 et reprise de session (`8fa3415`), la [CI](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40486263902), les [archives trois systèmes](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40486264176) et l’[essai prolongé trois nœuds](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40486263916) sont verts. Ce dernier a propagé 36 blocs en 481,9 secondes avec redémarrages. Le [fuzzing de cette révision](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40486263920) a aussi réussi sur les trois cibles bornées. Le lot suivant ajoute des tests de pair incohérent, des points de contrôle d’état, la borne d’horodatage SPV et des tests de reprise v6; sa [CI](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40488607637) a passé formatage, Clippy, compilation, tests Rust et smoke multi-nœuds. Les journaux confirment les six nouveaux tests; les [archives trois systèmes](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40488607543) ont aussi été construites. Le soak et le fuzzing de cette révision ont également terminé avec succès (liens ci-dessous). Aucun essai de volume supérieur à 100 000 blocs ne peut être revendiqué, et la copie de chaîne par page de sync reste coûteuse.

## Ce que ces essais ne valident pas

- Une saturation réelle du disque, une panne électrique, la durabilité après redémarrage matériel, ou une restauration interversion.
- Des essais sur des pairs hostiles, connexions lentes à grande échelle, partitions répétées ou un réseau multi-hôtes public pendant plusieurs jours. La [contre-revue technique](AI_COUNTER_REVIEW.md) a identifié des risques de temps futurs, de saturation des workers et de limite à 100 000 blocs; les correctifs partiels doivent encore être éprouvés à grande échelle. L’essai local de 490,2 secondes ne couvre pas ces cas.
- La résistance à une attaque d’éclipse contre le client SPV.
- Une revue indépendante du consensus, de la cryptographie, du portefeuille ou du réseau. Aucun reviewer n’est encore désigné.
- Une sécurité adaptée aux fonds réels. Les paramètres PoW sont faibles. Les clés nouvelles du wallet CLI sont chiffrées par défaut, mais son stockage, son UX et ses sauvegardes n’ont pas subi de revue indépendante.

Le plan et les portes restantes sont dans [ROADMAP_EXECUTION.md](ROADMAP_EXECUTION.md). Le dossier de revue est dans [INDEPENDENT_REVIEW.md](INDEPENDENT_REVIEW.md).

## Dépôt public et scan de l’historique — 27 octobre 2026

L’API GitHub confirme `private: false` et `visibility: public`. Un clone Git complet a récupéré toutes les branches, les tags disponibles et les références des PR 1 et 2. `git rev-parse --is-shallow-repository` retourne `false`. Au moment du scan, `git rev-list --all --count` recensait 487 commits accessibles; Gitleaks indique 479 commits effectivement examinés, pour environ 1,08 Mo de diffs.

Commande exécutée avec **Gitleaks 8.30.1**, binaire officiel Windows x64 dont le SHA-256 a été vérifié (`d29144deff3a68aa93ced33dddf84b7fdc26070add4aa0f4513094c8332afc4e`) :

```sh
gitleaks detect --source . --log-opts="--all" --redact=100 --ignore-gitleaks-allow --report-format json --report-path history-redacted.json --no-banner
```

Résultat : **code de sortie 0, aucune fuite signalée**, rapport JSON vide. Le sommet distant de travail était `1abe8ed30c9db7b770dd9e2afdf6344e4cb1e632`. Le scan utilise les règles de détection de Gitleaks; il ne prouve pas l’absence de secrets non reconnus, de références supprimées ou de fichiers jamais suivis. Aucune réécriture d’historique ni rotation de secret n’a été effectuée, puisqu’aucun secret n’a été signalé. Si un secret est découvert ultérieurement, le révoquer ou le remplacer avant de nettoyer l’historique.

## Dernières campagnes et limites

Sur `7d0f1ee598235b7194f92081424c008c4c628ceb`, la [CI](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40488607637), les [archives](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40488607543), le [soak](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40488607558) et le [fuzzing](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40488607519) sont terminés avec succès. Une contre-revue IA supplémentaire a échoué à démarrer en raison d’une limite d’usage; aucun résultat de revue ne peut lui être attribué.

L’essai de 72 heures n’a pas encore été exécuté. Le porteur a proposé son PC pour un essai local; plusieurs processus sur ce PC ne prouvent pas un fonctionnement sur deux ou trois machines. L’essai de volume supérieur à 100 000 blocs est également une preuve distincte, avec son workflow dédié.

## Reprise du 28 octobre 2026 — P1-4 (toolchain)

Avant : canal stable flottant dans le dépôt et sept workflows. Après : Rust 1.99.1 explicite dans le dépôt et les installations stables des workflows, y compris cargo-fuzz. Le moteur de fuzz conserve nightly, nécessaire aux sanitizers ; son épinglage reste à valider séparément. Version locale Ubuntu : rustc 1.99.1 (58cf8f9c3 2026-10-13). Validation locale Ubuntu réussie : cargo fmt --all -- --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo test --workspace --locked ; python3 scripts/testnet_smoke.py (wallet, SPV, 20 blocs, réorganisation vers hauteur 21, rattrapage, relais mempool, reprise après arrêt forcé hauteur 3 et limite écriture hauteur 2). Le test de volume 100 003 blocs reste ignoré dans cette suite et relève du workflow dédié. La CI distante de ce lot reste à exécuter après publication autorisée. cargo-deny 0.19.9 ajouté au workflow dependency-audit pour les licences et sources (configuration deny.toml : MIT, Apache-2.0, BSD-3-Clause, Unicode-3.0 ; registre crates.io uniquement, sources Git interdites). cargo-audit 0.22.2 conservé. cargo metadata --locked confirme ml-dsa 0.1.1, argon2 0.6.0, chacha20poly1305 0.11.0, getrandom 0.2.17 et rpassword 7.5.4, sans suffixe de préversion. Les versions 0.x ne constituent pas une garantie de stabilité API. Contrôle local cargo deny --locked check licenses sources : licenses ok, sources ok. Aucun changement de consensus.
