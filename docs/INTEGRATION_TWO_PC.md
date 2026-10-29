# Version commune pour les essais sur deux ordinateurs

Branche : integration/two-pc-testnet. Cette branche regroupe les PR #12 à #24 :
cahier des charges, clarification historique, auto-transferts, retarget,
admission P2P, cache de signatures, fuzzing des comptes, réorganisations,
réinsertion des paiements, saisie hexadécimale, délais RPC et partitions répétées.

Les sources exactes sont conservées comme parents de ce commit d'intégration.
Le fichier P2P combine explicitement le dispatcher de handshake de #16 avec le
cache et la publication atomique chaîne/mempool de #21. Les tests de #13 utilisent
désormais la reconstruction de mempool actuelle ; leurs assertions sont conservées.

## Vérification commune

La CI Rust et le smoke multi-nœuds doivent passer sur le commit intégré.
Les workflows retarget, admission, benchmark, fuzz des comptes et partitions sont
également exécutés sur la PR. Le workflow testnet-packages produit des archives
Windows, Linux et macOS, avec les sommes de contrôle existantes.

Pour les deux PC, utiliser le même artefact Windows du workflow testnet-packages
de cette branche et du même commit, avec des dossiers de données neufs.
Conserver le numéro de workflow et le SHA source. Ne pas mélanger une archive
de l'ancienne release avec celle de l'intégration.

Le protocole de manipulation est dans [TWO_PC_TEST.md](TWO_PC_TEST.md).
Comparer hauteur, hash du sommet et travail cumulé, puis noter le temps de
rattrapage après arrêt/reconnexion du pair.

Cette branche sert à tester l'ensemble. Les PR sources et les branches main et
feature/emission-schedule-v0 ne sont pas fusionnées par cette opération.
Les essais sur deux machines physiques restent à effectuer par leurs opérateurs.
