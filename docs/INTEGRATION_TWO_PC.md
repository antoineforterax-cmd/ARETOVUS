# Intégration AURORA — état et provenance pour un essai sur deux ordinateurs

Cette page remplace les notes historiques de la branche `integration/two-pc-testnet`.
**Au 30 octobre 2026**, `main` inclut les PR de consolidation jusqu'à la #76,
notamment le stockage v6, le P2P v4 et le test d'horloge simulée sur TCP.
Cette mention décrit la source du développement, **pas une validation physique**.
Une version exploitable pour l'essai doit être choisie par son **SHA complet**,
une **CI sur ce SHA réussie** et une **archive créée sur ce SHA**.

## Choisir les archives

Utiliser le [workflow testnet-packages](https://github.com/antoineforterax-cmd/AURORA/actions/workflows/testnet-packages.yml)
sur `main`, après succès des jobs Windows, Linux et macOS. Télécharger l'artefact
Windows **de ce même run**, extraire l'archive ZIP contenue dans l'artefact
si nécessaire, puis conserver `EXPERIMENTAL_TESTNET.txt` et `SHA256SUMS.txt`.

Comparer sur les deux machines le contenu de `Source commit:` ainsi que
les SHA-256 de `bin/aurora-node.exe` et `bin/aurora-wallet.exe`. Le script
`scripts/two_pc_preflight.ps1` vérifie les checksums internes et le démarrage
des exécutables ; il ne modifie ni les droits de pare-feu ni la configuration LAN.
Les archives CI sont non signées, expirent après 14 jours et restent
**expérimentales**. La release `testnet-v0.0.1-experimental.1` du 27 octobre
est basée sur un ancien commit et ne doit pas être mélangée avec `main`.

## Ordre d'exécution

Suivre [TWO_PC_TEST.md](TWO_PC_TEST.md) :
préparation de deux dossiers et deux clés indépendantes, connectivité
TCP privée, démarrage des serveurs, mineur dans un troisième dossier,
paiement d'essai confirmé, arrêt/reprise, convergence
`height/tip/work`, sauvegarde vérifiée et journal des incidents.
**Ne pas utiliser de véritables fonds ni de clés réelles.**

Aucun workflow GitHub exécuté sur un runner unique ne peut prouver
que deux machines physiques ont convergé. Les campagnes 72 h,
100k blocs sur TCP inter-hôtes, horloges réellement décalées,
disque plein et audit indépendant restent des jalons distincts.
