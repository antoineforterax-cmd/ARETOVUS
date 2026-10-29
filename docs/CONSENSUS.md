# Consensus du testnet AURORA

État du 27 octobre 2026, branche `feature/emission-schedule-v0`. Le code reste expérimental et non audité par un humain indépendant.

| Élément | Règle exécutée |
| --- | --- |
| Transactions | Profil v3 natif ML-DSA-65 sur Testnet; sérialisation canonique et domaines de signature; adresse de compte dérivée de la clé publique. Les profils v1 Ed25519 et v2 hybride sont limités à Regtest. |
| État | Montants entiers à huit décimales, nonce exact, solde et frais contrôlés avant mutation. Un transfert vers soi-même est rejeté. Le mempool existe mais ne fait pas partie du consensus. |
| Blocs | En-tête v1, racine Merkle des txids et engagement du mineur; PoW SHA-256 avec nombre requis de bits nuls en tête. Corps borné à 4 MiB moins la marge de transport. |
| Temps | Timestamp strictement supérieur au parent et à la médiane des onze ancêtres disponibles. Mineur, réseau et client SPV appliquent aussi une admission locale à au plus deux heures dans le futur; cette borne dépend de l’horloge locale et ne fait pas partie du rejeu du consensus. |
| Difficulté | 8 bits initialement; cible 60 secondes, fenêtre de 10 blocs, retarget entier borné à deux bits, minimum 4 et maximum 20. Le candidat ferme la fenêtre mesurée depuis l’ancêtre H−10. |
| Sélection | Travail `2^difficulty_bits` accumulé sur 320 bits; plus grand travail, puis plus petit hash en cas d’égalité. |
| Émission | Genèse vide; plafond brut de 500 millions AUR; `min(reliquat, max(1, floor(reliquat / 1_000_000)))`. Premier bloc : 500 AUR. Le burn ne réouvre pas le plafond. |
| Répartition | 5 % de l’émission à la trésorerie fixe `[0xA0; 32]`, reliquat au mineur. Moitié entière des frais au mineur, reliquat détruit. À plafond atteint, aucune nouvelle émission. |
| Réorganisation | Validation contre l’état du parent, avec récompenses et frais propres à chaque branche. Le sommet canonique et des points de contrôle tous les 256 blocs sont conservés; les autres états sont rejoués. Aucun horizon de réorganisation n’est ajouté par ce cache. |
| Stockage | Blocs immuables, pages et manifestes checksummés v6, trois points de reprise; lecture des snapshots v4 et index v5, puis revalidation. |
| Transport | Fingerprint P2P v4, inventaires u64 en ordre d’arrivée, pages bornées, budgets de session et contrôle de continuité à la reconnexion. Les limites réseau ne remplacent jamais la validation consensus. |

Le compte de trésorerie n’est pas un contrat gouverné. DID, gardiens, récupération et recyclage des fonds inactifs ne sont pas activés. Il n’existe pas de maturité différée des récompenses. Le profil SHA-256/PoW n’est pas présenté comme entièrement post-quantique.

Les tests couvrent signatures, montants, nonce, atomicité, Merkle, retarget, travail, réorganisations et récompenses, stockage et réseau. Les preuves sont rattachées à des commits dans [TESTNET_VALIDATION_REPORT.md](TESTNET_VALIDATION_REPORT.md). Les coûts de rejeu d’état, la mémoire, les pairs hostiles et le comportement sur plusieurs machines pendant 72 heures restent à mesurer. Les formats détaillés sont dans [PROTOCOL.md](PROTOCOL.md).
