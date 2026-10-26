# Rapport d’essais du testnet AURORA

**Commit de code et CI multi-nœuds :** `625084b953f1101fb6d0f168e5dd0e0417fa93d3`  
**Date des workflows :** 26 octobre 2026  
**Branche :** `feature/emission-schedule-v0`  
**Verdict :** les contrôles automatisés ciblés passent; ce résultat ne constitue ni un audit indépendant ni une garantie pour des fonds réels.

## Contrôles de compilation et tests

- [CI du verrou de dossier et du smoke test séparant mineur et serveur](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40361926990) : formatage, Clippy avec avertissements refusés, compilation workspace, tests et smoke test testnet — succès.
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
8. Arrête brutalement un mineur après plusieurs index validés, puis vérifie qu’il redémarre au-dessus du dernier index complet.
9. Sur Linux, impose une limite de taille aux écritures pour simuler un échec de sauvegarde; vérifie que le nœud reprend l’index complet précédent. Les tests Rust restaurent aussi une copie complète du dossier v5 et lisent un snapshot v4 avant écriture v5.

Le journal CI résume les assertions : « wallet, SPV, 20-block soak, heavier-branch reorganization, live peer catch-up, mempool relay, forced-stop and write-limit recovery passed ». Résultat : succès sur le commit indiqué.

## Essai prolongé et archives testnet

- [Essai à trois nœuds avec verrou de dossier](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40361895189) : 36 blocs propagés et vérifiés sur trois processus en **490,2 secondes**; le pair intermédiaire a été arrêté puis redémarré quatre fois, et une copie complète de son dossier a été restaurée et vérifiée au même sommet — succès. Cet essai est local au runner CI; il ne prouve pas un fonctionnement continu sur plusieurs hôtes ou pendant des jours.
- [Construction des archives avec verrou de dossier](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40361894839) : compilation Rust release et empaquetage du nœud et du wallet réussis sur Windows, Linux et macOS. Chaque archive contient les binaires, la documentation et `SHA256SUMS.txt`, vérifié pendant l’empaquetage. Ces artefacts privés non signés expirent après 14 jours; aucun installateur ni publication publique n’existe encore.

## Essai manuel fourni par le porteur

Le 26 octobre 2026, le porteur a exécuté sur Windows une archive testnet construite par CI (révision `237e87d9a33c`). La sortie PowerShell fournie montre : refus d’une confirmation de mot de passe différente, création d’une clé chiffrée `AURKEY4`, minage avec le même dossier de données sur plusieurs invocations jusqu’à la hauteur **367** (vérification locale explicitement réussie à 311; sortie de minage fournie jusqu’à 367), et démarrage du serveur sur `127.0.0.1:18444`. Le wallet a lu un solde confirmé de **1 424,99857500 AUR** sur le serveur lancé à la hauteur 3; un second dossier a synchronisé ces 3 blocs puis passé `testnet verify` (4 blocs connus avec la genèse). Les retargets observés sur les premières invocations suivent 8→6→8→10→12→14 bits. Une seconde tentative de `serve` sur le même port a été refusée par Windows avec l’erreur 10048; le port était déjà occupé. Le serveur doit être arrêté puis relancé pour charger les blocs minés ultérieurement; une nouvelle commande de synchronisation a accepté **0** bloc et le second dossier est resté à 3, car l’ancien serveur occupait toujours le port. La synchronisation jusqu’à 367 n’est pas encore démontrée. L’adresse et le chemin locaux ne sont pas recopiés ici. Le porteur rapporte aussi que les binaires macOS et Linux démarrent sur d’autres PC; aucun journal de synchronisation entre ces machines n’a été fourni. Ce retour manuel ne remplace pas un essai réseau multi-hôtes vérifié.

## Campagnes fuzz

Les trois cibles ont été exécutées pendant 301 secondes chacune sur le commit `31b016e931624785508cf82b464efa19557c31bf` :

- `canonical_decode` : **52 129 549** entrées sur les décodeurs canoniques transaction, compte/bloc et en-tête.
- `merkle_proofs` : **55 443 335** entrées sur le vérificateur de preuves Merkle.
- `network_payloads` : **60 256 094** entrées sur les parseurs de charges P2P, blocs et listes de pairs.

Aucun crash n’a été rapporté pendant ces campagnes. Les journaux de campagne sont dans le workflow lié ci-dessus. Ces essais CI sont bornés; ils ne remplacent pas un fuzzing prolongé avec corpus conservé et revue humaine indépendante.

## Ce que ces essais ne valident pas

- Une saturation réelle du disque, une panne électrique, la durabilité après redémarrage matériel, ou une restauration interversion.
- Des essais sur des pairs hostiles, connexions lentes à grande échelle, partitions répétées ou un réseau multi-hôtes public pendant plusieurs jours. L’essai local de 490,2 secondes ne couvre pas ces cas.
- La résistance à une attaque d’éclipse contre le client SPV.
- Une revue indépendante du consensus, de la cryptographie, du portefeuille ou du réseau. Aucun reviewer n’est encore désigné.
- Une sécurité adaptée aux fonds réels. Les paramètres PoW sont faibles. Les clés nouvelles du wallet CLI sont chiffrées par défaut, mais son stockage, son UX et ses sauvegardes n’ont pas subi de revue indépendante.

Le plan et les portes restantes sont dans [ROADMAP_EXECUTION.md](ROADMAP_EXECUTION.md). Le dossier de revue est dans [INDEPENDENT_REVIEW.md](INDEPENDENT_REVIEW.md).
