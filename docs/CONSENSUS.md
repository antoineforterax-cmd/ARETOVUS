# CONSENSUS.md — AURORA

**Statut : prototype expérimental, aucun paramètre économique ou de consensus final n’est approuvé.** Le code a des règles exécutables, mais elles ne constituent pas encore un consensus complet ni un réseau utilisable avec des fonds.

## Règles exécutées actuellement

| Sujet | Règle de prototype | Limite |
|---|---|---|
| Encodage | Entiers big-endian, longueurs bornées, octets excédentaires rejetés | Format versionné expérimental |
| Transfert | Signature Ed25519 stricte, identifiant réseau, nonce, solde et frais validés | Pas de mempool ni de règle de frais minimum |
| Bloc compte | Engagement ordonné de txids, validation d’état atomique, pas de Merkle | Aucun reward/coinbase |
| PoW | SHA-256 avec nombre minimal de bits nuls; retarget expérimental à fenêtres, pas entiers bornés, horodatage supérieur au temps médian des 11 derniers blocs | Paramètres par chaîne; règle non simulée/auditée |
| Sélection de branche | Plus grand travail cumulé, travail par bloc `2^difficulty_bits`; égalité départagée par la plus petite empreinte | Comptabilité u320; aucune simulation adversariale |
| Retarget | Chaque fenêtre compare les médianes de temps à la durée cible; difficulté ajustée par pas entiers d’un bit, limite par fenêtre et bornes min/max | Règle candidate, distincte d’ASERT; vecteurs et analyse anti-manipulation à approfondir |
| Stockage | Snapshot version 1, SHA-256, relecture des blocs et validation au chargement | Branche canonique seulement; état initial fourni par le profil réseau |
| Testnet locale | `aurora-node regtest` avec difficulté 0 | Pas de P2P ni de minage |

## Règles qui manquent pour un consensus testnet complet

1. Simuler la règle de difficulté candidate, vérifier les retargets longs/courts, les manipulations temporelles et les égalités; la comparer à ASERT et figer des vecteurs indépendants avant testnet.
2. Calendrier de récompense approuvé, création atomique, frais/burn/trésorerie, plafond d’offre et limites par réorganisation.
3. Horodatages, validation temporelle, bornes d’en-tête et politique des blocs obsolètes.
4. Génèse et état initial reproductibles avec identifiant de chaîne non réutilisé.
5. Protocole P2P versionné, limites anti-DoS, synchronisation, annonces et reconnexion.
6. Persistance de branches/état, reprise après écriture interrompue, migration, sauvegarde et récupération.

## Invariants obligatoires

- Même chaîne valide et même paramètres produisent la même difficulté, état, récompense et tête canonique.
- Aucun état partiel après rejet d’une transaction ou d’un bloc.
- L’offre ne dépasse jamais la règle d’émission approuvée, même après réorganisation.
- La chaîne active est celle du plus grand travail cumulé une fois le PoW variable spécifié.
- Un encodage accepté est exactement l’encodage canonique reçu.
- Les données d’un réseau ne sont pas valides sur un autre.

Les règles ci-dessus doivent avoir une spécification et des vecteurs indépendants avant une testnet publique. La CI confirme uniquement les contrôles automatisés du code soumis, pas la sécurité du protocole.
