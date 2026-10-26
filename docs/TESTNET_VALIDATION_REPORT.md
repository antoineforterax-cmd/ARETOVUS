# Rapport d’essais du testnet AURORA

**Commit de code et CI multi-nœuds :** `23e89811a0c16834472607f6ef2ae8afe5840546`  
**Date des workflows :** 26 octobre 2026  
**Branche :** `feature/emission-schedule-v0`  
**Verdict :** les contrôles automatisés ciblés passent; ce résultat ne constitue ni un audit indépendant ni une garantie pour des fonds réels.

## Contrôles de compilation et tests

- [CI de la révision réseau avec libération des verrous avant envoi](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40360678457) : formatage, Clippy avec avertissements refusés, compilation workspace, tests et smoke test testnet — succès.
- [CI de l’ajout du rattrapage automatique](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40333338987) : mêmes contrôles, y compris le scénario d’un pair en fonctionnement qui rejoint les hauteurs 21 puis 22 — succès.
- [Campagne libFuzzer](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40326204232) sur le commit `390eb1d9517ffd61f560ba5dff065a4c9136c36f` : trois cibles exécutées 301 secondes chacune — succès, sans crash rapporté. Les changements de stockage et de rattrapage ultérieurs sont couverts par les tests/CI courants; cette campagne fuzz ne les couvre pas directement.

## Scénario multi-nœuds

Le smoke test utilise plusieurs processus isolés et des dossiers temporaires :

1. Mine quatre blocs et vérifie l’émission ainsi que la répartition 95 % mineur / 5 % trésorerie.
2. Crée et utilise un wallet testnet, vérifie son entrée au mempool, son inclusion et sa preuve d’inclusion Merkle avec un minimum de confirmations.
3. Synchronise les nœuds, redémarre un serveur depuis son snapshot et fait télécharger le pair frais.
4. Mine jusqu’à la hauteur 20, couvrant deux retargets, pousse cette chaîne, redémarre le nœud receveur et vérifie la synchronisation de 16 blocs.
5. Crée une branche concurrente de hauteur 21 depuis la genèse, vérifie que son travail cumulé provoque la réorganisation du nœud, puis vérifie la convergence d’un autre pair. La transaction devenue orpheline n’est plus rapportée comme confirmée.
6. Lance un pair avec un serveur déjà actif, vérifie qu’il rejoint automatiquement une branche de hauteur 21, puis qu’il rattrape un nouveau bloc de hauteur 22 sans commande `sync` manuelle.
7. Soumet une nouvelle transaction à un pair, vérifie son relais périodique à un second pair, puis la fait miner par un nœud qui ne contacte que le second pair.
8. Arrête brutalement un mineur après plusieurs index validés, puis vérifie qu’il redémarre au-dessus du dernier index complet.
9. Sur Linux, impose une limite de taille aux écritures pour simuler un échec de sauvegarde; vérifie que le nœud reprend l’index complet précédent. Les tests Rust restaurent aussi une copie complète du dossier v5 et lisent un snapshot v4 avant écriture v5.

Le journal CI résume les assertions : « wallet, SPV, 20-block soak, heavier-branch reorganization, live peer catch-up, forced-stop and write-limit recovery passed ». Résultat : succès sur le commit indiqué.

## Essai prolongé et archives testnet

- [Essai à trois nœuds](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40357713892) : 36 blocs propagés et vérifiés sur trois processus en **419,1 secondes**; le pair intermédiaire a été arrêté puis redémarré quatre fois, et une copie complète de son dossier a été restaurée et vérifiée au même sommet — succès. Cet essai est local au runner CI; il ne prouve pas un fonctionnement continu sur plusieurs hôtes ou pendant des jours.
- [Construction des archives](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40360682294) : compilation Rust release et empaquetage du nœud et du wallet réussis sur Windows, Linux et macOS. Chaque archive contient les binaires, la documentation et `SHA256SUMS.txt`, vérifié pendant l’empaquetage. Ces artefacts privés non signés expirent après 14 jours; aucun installateur ni publication publique n’existe encore.

## Essai manuel fourni par le porteur

Le 26 octobre 2026, le porteur a exécuté sur Windows une archive testnet construite par CI (révision `237e87d9a33c`). La sortie PowerShell fournie montre : refus d’une confirmation de mot de passe différente, création d’une clé chiffrée `AURKEY4`, minage avec le même dossier de données sur plusieurs invocations jusqu’à la hauteur **58**, et démarrage du serveur sur `127.0.0.1:18444`. Les retargets observés suivent 8→6→8→10→12→14 bits. L’adresse et le chemin locaux ne sont pas recopiés ici. Le porteur rapporte aussi que les binaires macOS et Linux démarrent sur d’autres PC; aucun journal de synchronisation entre ces machines n’a été fourni. Ce retour manuel ne remplace pas un essai réseau multi-hôtes vérifié.

## Campagnes fuzz

Les trois cibles ont été exécutées pendant 301 secondes chacune sur le commit `390eb1d9517ffd61f560ba5dff065a4c9136c36f` :

- `canonical_decode` : **36 029 045** entrées sur les décodeurs canoniques transaction, compte/bloc et en-tête.
- `merkle_proofs` : **42 716 216** entrées sur le vérificateur de preuves Merkle.
- `network_payloads` : **33 451 840** entrées sur les parseurs de charges P2P, blocs et listes de pairs.

Aucun crash n’a été rapporté pendant ces campagnes. Les journaux de campagne sont dans le workflow lié ci-dessus. Ces essais CI sont bornés; ils ne remplacent pas un fuzzing prolongé avec corpus conservé et revue humaine indépendante.

## Ce que ces essais ne valident pas

- Une saturation réelle du disque, une panne électrique, la durabilité après redémarrage matériel, ou une restauration interversion.
- Des essais sur des pairs hostiles, connexions lentes à grande échelle, partitions répétées ou un réseau multi-hôtes public pendant plusieurs jours. L’essai local de 419,1 secondes ne couvre pas ces cas.
- La résistance à une attaque d’éclipse contre le client SPV.
- Une revue indépendante du consensus, de la cryptographie, du portefeuille ou du réseau. Aucun reviewer n’est encore désigné.
- Une sécurité adaptée aux fonds réels. Les paramètres PoW sont faibles. Les clés nouvelles du wallet CLI sont chiffrées par défaut, mais son stockage, son UX et ses sauvegardes n’ont pas subi de revue indépendante.

Le plan et les portes restantes sont dans [ROADMAP_EXECUTION.md](ROADMAP_EXECUTION.md). Le dossier de revue est dans [INDEPENDENT_REVIEW.md](INDEPENDENT_REVIEW.md).
