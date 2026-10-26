# Rapport d’essais automatisés du testnet AURORA

**Commit de code et CI multi-nœuds :** `b8a7626f1f8ad2a8018687783153601249123c8b`  
**Date des workflows :** 26 octobre 2026  
**Branche :** `feature/emission-schedule-v0`  
**Verdict :** les contrôles automatisés ciblés passent; ce résultat ne constitue ni un audit indépendant ni une garantie pour des fonds réels.

## Contrôles de compilation et tests

- [CI de la révision courante](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40333486589) : formatage, Clippy avec avertissements refusés, compilation workspace, tests et smoke test testnet — succès.
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
7. Arrête brutalement un mineur après plusieurs index validés, puis vérifie qu’il redémarre au-dessus du dernier index complet.
8. Sur Linux, impose une limite de taille aux écritures pour simuler un échec de sauvegarde; vérifie que le nœud reprend l’index complet précédent. Les tests Rust restaurent aussi une copie complète du dossier v5 et lisent un snapshot v4 avant écriture v5.

Le journal CI résume les assertions : « wallet, SPV, 20-block soak, heavier-branch reorganization, live peer catch-up, forced-stop and write-limit recovery passed ». Résultat : succès sur le commit indiqué.

## Campagnes fuzz

Les trois cibles ont été exécutées pendant 301 secondes chacune sur le commit indiqué :

- `canonical_decode` : **36 029 045** entrées sur les décodeurs canoniques transaction, compte/bloc et en-tête.
- `merkle_proofs` : **42 716 216** entrées sur le vérificateur de preuves Merkle.
- `network_payloads` : **33 451 840** entrées sur les parseurs de charges P2P, blocs et listes de pairs.

Aucun crash n’a été rapporté pendant ces campagnes. Les journaux de campagne sont dans le workflow lié ci-dessus. Ces essais CI sont bornés; ils ne remplacent pas un fuzzing prolongé avec corpus conservé et revue humaine indépendante.

## Ce que ces essais ne valident pas

- Une saturation réelle du disque, une panne électrique, la durabilité après redémarrage matériel, ou une restauration interversion.
- Une campagne longue sur des pairs hostiles, connexions lentes à grande échelle, partitions répétées ou réseau multi-hôtes public.
- La résistance à une attaque d’éclipse contre le client SPV.
- Une revue indépendante du consensus, de la cryptographie, du portefeuille ou du réseau. Aucun reviewer n’est encore désigné.
- Une sécurité adaptée aux fonds réels. Les paramètres PoW sont faibles et les clés du wallet CLI sont en clair.

Le plan et les portes restantes sont dans [ROADMAP_EXECUTION.md](ROADMAP_EXECUTION.md). Le dossier de revue est dans [INDEPENDENT_REVIEW.md](INDEPENDENT_REVIEW.md).
