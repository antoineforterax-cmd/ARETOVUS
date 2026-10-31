# 7. Technologie, limites et prochaines étapes

[Sommaire](README.md) · [← Exploitation](06-exploitation.md)

## La version de référence

| Partie | Choix exécuté au commit de référence |
| --- | --- |
| Langage | Rust, workspace de cinq crates. |
| Transactions | Modèle par comptes, encodage canonique v3, nonce, montants entiers et vérifications avant mutation. |
| Signatures | ML-DSA-65 natif pour Testnet, dépendance RustCrypto `ml-dsa` épinglée à `0.1.1`. Les profils Ed25519 et hybrides sont limités à Regtest. |
| Adresse | Empreinte avec séparation de domaine de la clé publique, sur 32 octets, affichée en 64 caractères hexadécimaux. |
| Blocs | En-tête v1, engagement du parent, du mineur et racine Merkle des transactions. |
| Preuve de travail | SHA-256 et exigence d’un nombre de bits nuls en tête; ce n’est pas un PoW annoncé résistant aux ASIC. |
| Difficulté | Cible de 60 secondes, fenêtre de 10 blocs, départ à 8 bits, bornes de 4 à 20 bits et variation maximale de deux bits par fenêtre. |
| Horodatages | Strictement croissants et supérieurs à la médiane des onze ancêtres disponibles; admission locale bornée à deux heures dans le futur. |
| Choix de chaîne | Travail cumulé, puis règle déterministe d’égalité; les réorganisations rejouent les règles de chaque branche. |
| Stockage | Fichiers locaux immuables de blocs, pages d’index v6 et manifestes checksummés; lecture des anciens formats v4/v5 compatibles. |
| Réseau | Protocole TCP propre à ARETOVUS, fingerprint P2P v4, pairs configurés, pages et quotas bornés, rattrapage périodique. |
| Wallet | CLI, fichiers chiffrés, sauvegarde/restauration, preuves Merkle et vérification légère SPV. |

La [carte des cinq crates](../../ARCHITECTURE.md) aide à retrouver les modules. Le [protocole](../PROTOCOL.md) précise les formats et limites; cette page en résume les choix, sans proposer une autre spécification.

## Post-quantique : ce que cela couvre

ML-DSA-65 est le schéma de signature choisi pour les transactions. L’autorisation ne dépend plus d’une signature Ed25519 dans le profil Testnet. La clé publique occupe 1 952 octets et la signature 3 309 octets, soit 5 261 octets pour ces deux éléments, avant les autres champs de la transaction.

Le choix d’un schéma standardisé ne certifie pas son implémentation ni son intégration. Les vecteurs normatifs indépendants, la génération des clés, leur stockage, les canaux auxiliaires et les règles de validation doivent aussi être examinés. Le hachage, le PoW et le réseau restent des sujets distincts.

Aucune protection mathématique absolue pour les trente prochaines années n’est annoncée. Il faut conserver une capacité d’évolution du protocole et définir comment une modification serait activée. Remplacer immédiatement ML-DSA-65 par un autre schéma sans besoin démontré ajouterait une nouvelle migration et une nouvelle surface à vérifier.

## Autonomie et P2P

Le nœud valide et conserve les données localement. Il ne demande pas de service de base de données externe ni de fournisseur d’API pour chaque paiement. Le réseau actuel nécessite toutefois des adresses initiales de pairs joignables et une configuration cohérente.

**libp2p, Kademlia DHT, relais automatiques et traversée générale des pare-feux ne sont pas intégrés à cette version.** Ce sont des pistes de développement. Les adopter exige de mesurer les dépendances, la mémoire, les connexions, l’identité des pairs et la résistance aux attaques d’éclipse et de saturation.

Une découverte distribuée ne crée pas une route réseau à travers tout pare-feu et ne garantit pas la survie après la déconnexion de 90 % des nœuds. Le résultat dépend de la topologie, des nœuds restants, de leur joignabilité et de la puissance de minage disponible. Les tests devront préciser les scénarios qu’ils démontrent.

## Consensus et résistance aux ASIC

**ASERT n’est pas intégré.** La difficulté courante utilise un nombre entier de bits nuls et des ajustements par fenêtre. Passer à une cible plus fine touche au PoW, au calcul du travail, au client léger et potentiellement aux formats ou à l’empreinte de protocole. Ce changement demande une spécification, des vecteurs indépendants et des essais de réorganisation avant activation sur un nouveau profil.

Un PoW visant à réduire l’avantage du matériel spécialisé doit être étudié à partir de constructions existantes et de mesures. Ce guide ne présente pas le SHA-256 courant comme résistant aux ASIC et ne propose pas d’algorithme cryptographique inventé pour ARETOVUS.

L’accessibilité d’un mineur sur PC ne suffit pas à établir une décentralisation durable. Les coûts d’énergie, la distribution de puissance, les règles d’admission et les risques de manipulation temporelle restent à mesurer.

## Stockage et programme autonome

Le stockage actuel n’utilise ni Sled ni RocksDB. Le nœud écrit ses propres fichiers dans son dossier. Cela évite l’installation d’un serveur de base de données, mais laisse davantage de logique de stockage à tester et auditer.

Un exécutable autonome n’est pas isolé du système comme une machine virtuelle : il utilise des ressources, des fichiers, le réseau et les permissions de son utilisateur. Il faut borner ces usages et tester leurs défaillances. Ajouter une base embarquée ne garantit pas, à lui seul, la durabilité ni la sécurité.

## Ce qui reste à prouver

Les contrôles CI vérifient des propriétés précises sur des commits précis. La [campagne de volume du commit `421d443`](https://github.com/antoineforterax-cmd/ARETOVUS/actions/runs/40527921129) a testé un stockage contenant 100 003 blocs connus, avec une branche secondaire et rechargement. Ce scénario n’est pas une synchronisation réseau de 100 000 blocs et n’est pas un essai de panne électrique.

Les travaux suivants restent distincts : endurance avec transactions et mesures, scénarios de pairs hostiles, longues réorganisations, données corrompues, disque réellement plein, restauration hors machine, simulation indépendante du retarget, vecteurs cryptographiques et revue humaine indépendante. La [fiche de finalisation](../FINALIZATION_CHECKLIST.md) et le [rapport d’essais](../TESTNET_VALIDATION_REPORT.md) suivent les preuves disponibles et les tâches ouvertes.

Les DID, gardiens, délais et fonds inactifs restent au stade de conception. La trésorerie n’a pas de voie de dépense. Les interfaces graphiques, l’explorateur, le site, la vente ou l’échange et le mobile sont reportés pendant le travail sur le cœur. Ils ne doivent pas être annoncés comme disponibles dans une présentation du projet.

## Participer sans surpromettre

Un compte rendu utile décrit le problème, le choix retenu, le commit, le test exécuté, le résultat et ses limites. Un journal de développement peut publier ces faits même sans participants; il n’a pas besoin d’inventer une communauté, un audit ou des échanges humains.

Les contributions sont proposées sous la double licence MIT ou Apache-2.0. Une relecture humaine ciblée, un test qui échoue de façon reproductible ou une mesure de charge sont utiles au projet. Les [instructions de contribution](../../CONTRIBUTING.md), la [provenance](../../PROVENANCE.md) et le [dossier de revue](../INDEPENDENT_REVIEW.md) expliquent comment travailler sur cette base expérimentale.
