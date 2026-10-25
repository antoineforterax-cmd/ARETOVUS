# AURORA — règles de consensus expérimentales

**Statut : testnet expérimentale locale, non auditée.** Les règles ci-dessous décrivent le code de `feature/emission-schedule-v0`; elles ne constituent ni un lancement public ni une garantie de sécurité mainnet.

## Profil de chaîne exécuté

| Sujet | Règle codée | Limite restante |
|---|---|---|
| Transactions | Encodage canonique; Ed25519 stricte; clé publique 32 octets comme compte; nonce exact; solde/frais validés atomiquement | Pas d’adresse texte, mempool ou frais minimums |
| Blocs | En-tête version 0; engagement ordonné des txids et de l’identifiant mineur; SHA-256 avec bits nuls en tête | Engagement de liste, pas un arbre de Merkle; limite consensus de 4 MiB |
| Temps | Timestamps strictement supérieurs au parent et à la médiane des 11 ancêtres disponibles | Pas de borne future définie par consensus |
| Difficulté | Retarget candidat-dépendant sur le delta brut `timestamp_candidat - timestamp(H - intervalle)`, pas entiers bornés min/max | Simulations temporelles indépendantes encore nécessaires |
| Travail | `2^difficulty_bits` par bloc dans un accumulateur u320; chaîne au plus grand travail, égalité départagée par le hash le plus petit | Pas de revue externe |
| Récompense | Profil `fast_start_500`; plafond brut 500 M AUR, paliers et split décrits dans `EMISSION_MODEL.md` et le code | Émission basée sur l’offre courante, testnet seulement |
| Réorganisation | État et récompenses sont calculés depuis l’état du parent de chaque branche; la branche au travail supérieur devient canonique | Pas de maturité de coinbase ni dépense de récompense différée |
| Stockage | Snapshot version 3, checksum, paramètres difficulté/récompense et toutes les branches; rejeu/revalidation à la lecture | Pas de journal transactionnel, migration de format ou test complet de coupure disque |
| Réseau | TCP expérimental push/pull vers pairs configurés, fingerprint paramètres/genèse, frames bornées, budget de 256 MiB/16 384 requêtes/15 minutes par session, pages de blocs et validation consensus | Pas de découverte automatique, propagation spontanée, connexions simultanées, reconnexion ou suite DoS complète |

Les constructeurs de chaîne génériques désactivent les récompenses par défaut. Le nœud testnet choisit explicitement le profil de récompense et la difficulté initiale. Le fingerprint P2P comprend le hash de genèse, la difficulté, le retarget, le drapeau des récompenses et le compte de trésorerie.

## Profil testnet local

- Réseau : `NetworkId::Testnet`, genèse déterministe et vide.
- Difficulté initiale : 8 bits nuls en tête.
- Cible : 60 secondes; fenêtre : 10 blocs; ajustement entier borné à 2 bits par fenêtre; bornes de 4 à 20 bits.
- Trésorerie : compte fixe de test `[0xA0; 32]`.
- Les blocs sont minés par `aurora-node testnet mine`, sauvegardés après chaque bloc et revalidés à la reprise.
- Synchronisation via `aurora-node testnet serve` et `aurora-node testnet sync`; seul le téléchargement pull est fourni.

## Invariants couverts par tests

- Même parent, bloc et paramètres donnent le même état validé; transaction ou bloc invalide ne produit pas d’état partiel.
- Le plafond brut n’est pas dépassé, même au dernier palier; les frais brûlés ne réouvrent pas le plafond.
- Le compte du mineur est inclus dans l’engagement du bloc.
- Une branche secondaire garde son propre état et ses récompenses; une réorganisation change l’état canonique.
- Un snapshot conserve les branches et les paramètres, puis rejoue les blocs au chargement.
- La synchronisation rejette les fingerprints différents et revalide les blocs reçus avant sauvegarde.

Le format de transaction v2 requiert Ed25519 et ML-DSA-65 simultanément; il n’active aucune migration de comptes. La bibliothèque RustCrypto ML-DSA n’a pas d’audit indépendant. Il s’agit d’un format de test uniquement.

La CI verte confirme seulement les contrôles automatisés. Avant de qualifier la chaîne de testnet multi-nœud utilisable, il faut compléter les essais de temps et retarget, les limites de taille, les pairs hostiles, la propagation et la reconnexion, les coupures de stockage et le fuzzing. Une revue indépendante est nécessaire avant tout usage au-delà d’expériences locales.
