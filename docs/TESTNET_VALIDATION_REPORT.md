# Rapport d’essais automatisés du testnet AURORA

**Commit testé :** `05f3e49505acc879bd9599ed0a6af6a445e3d4a0`  
**Date des workflows :** 26 octobre 2026  
**Branche :** `feature/emission-schedule-v0`  
**Verdict :** les contrôles automatisés ciblés passent; le résultat n’est ni un audit indépendant, ni une garantie pour des fonds réels.

## Contrôles de compilation et tests

- [CI multi-processus 1](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40325271969) : formatage, Clippy avec avertissements refusés, compilation workspace, tests et smoke test testnet — succès.
- [CI multi-processus 2](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40325263850) : mêmes contrôles sur le commit — succès.
- [Campagnes libFuzzer](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40325272093) : deux cibles, chacune exécutée 31 secondes — succès.

## Scénario multi-nœuds

Le smoke test utilise plusieurs processus isolés et des dossiers temporaires :

1. Mine quatre blocs et vérifie l’émission ainsi que la répartition 95 % mineur / 5 % trésorerie.
2. Crée et utilise un wallet testnet, vérifie son entrée au mempool, son inclusion et sa preuve d’inclusion Merkle avec un minimum de confirmations.
3. Synchronise les nœuds, redémarre un serveur depuis son snapshot et fait télécharger le pair frais.
4. Mine jusqu’à la hauteur 20, couvrant deux retargets, pousse cette chaîne, redémarre le nœud receveur et vérifie la synchronisation de 16 blocs.
5. Crée une branche concurrente de hauteur 21 depuis la genèse, vérifie que son travail cumulé provoque la réorganisation du nœud, puis vérifie la convergence d’un autre pair. La transaction devenue orpheline n’est plus rapportée comme confirmée.
6. Arrête brutalement un mineur après plusieurs snapshots validés, puis vérifie qu’il redémarre au-dessus du dernier snapshot complet.
7. Sur Linux, impose une limite de taille aux écritures pour simuler un échec de sauvegarde; vérifie que le nœud reprend le snapshot complet précédent.

Résultat du smoke test : succès sur le commit indiqué.

## Campagnes fuzz

- `canonical_decode` : 5 243 706 entrées exécutées en 31 secondes sur les décodeurs canoniques de transaction, bloc et en-tête; aucun crash rapporté.
- `merkle_proofs` : 5 748 834 entrées exécutées en 31 secondes sur le vérificateur de preuves; aucun crash rapporté.

Il s’agit de courtes campagnes CI. Elles ne remplacent pas des fuzzings prolongés, une couverture mesurée, des corpus conservés ni une revue humaine indépendante.

## Ce que ces essais ne valident pas

- Une saturation réelle du disque, une panne électrique, la durabilité après redémarrage matériel, ou une restauration interversion.
- Une campagne longue sur des pairs hostiles, connexions lentes à grande échelle, partitions répétées ou réseau multi-hôtes public.
- La résistance à une attaque d’éclipse contre le client SPV.
- Une revue indépendante du consensus, de la cryptographie, du portefeuille ou du réseau.
- Une sécurité adaptée aux fonds réels. Les paramètres PoW sont faibles et les clés du wallet CLI sont en clair.

Le plan et les portes restantes sont dans [ROADMAP_EXECUTION.md](ROADMAP_EXECUTION.md). Le dossier de revue est dans [INDEPENDENT_REVIEW.md](INDEPENDENT_REVIEW.md).
