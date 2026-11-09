# RFC — conserver SHA-256, étudier un PoW mémoire sans changer le consensus

**Statut : PROPOSITION, non approuvée et non implémentée.** Référence initiale : `main@caa94228b2fb769e3a5ea479ea5fd07485744489` (9 novembre 2026). Cette RFC ne modifie **aucune** règle de consensus, genèse, récompense ni format réseau. L'utilisateur souhaite conserver SHA-256 tout en limitant les risques d'attaque contre une future petite blockchain.

## Ce que « attaque de SHA » peut vouloir dire

1. **Casser SHA-256 cryptographiquement** : produire une collision ou une préimage exploitable. Ne pas confondre cette question avec la taille du réseau. SHA-256 reste une primitive largement utilisée ; cette RFC **ne revendique pas une sécurité éternelle**.
2. **Attaquer un PoW SHA-256** : concentrer/louer de la puissance de calcul, produire une branche concurrente, censurer ou réorganiser des transactions. **Ce risque existe même si SHA-256 n'est pas cassé.**
3. **Exploiter le protocole** : un défaut de retarget, de validation, de sélection de chaîne, de timestamp ou de P2P peut être exploité quel que soit le hash.

## Décision de conception proposée

- **Conserver SHA-256** pour les identifiants, engagements, domaines de hachage et fonctions où il est déjà une primitive du testnet. Ne pas changer ces composants pour suivre une mode.
- **Conserver le PoW SHA-256 actif** jusqu'à décision expresse et tests. Il reste aussi un profil expérimental facile à reproduire en Regtest.
- **Évaluer Argon2id séparément** comme candidat de PoW mémoire. La présence de la crate `argon2` dans le wallet prouve seulement son emploi comme dépendance du wallet : **elle ne constitue pas un audit d'un PoW de consensus**.
- **Envisager un profil versionné** `SHA256` ou `Argon2id` (ou une composition définie explicitement) pour faciliter l'expérimentation. Ne jamais choisir l'algorithme selon une valeur reçue d'un pair sans vérification de l'identité du réseau, de l'activation et du fingerprint. Pas de bascule automatique.
- Si le PoW change pour un nouveau testnet, **nouvelle identité de chaîne, genèse, fingerprint, vecteurs, règle de difficulté, jeux de données et tests**. Ne pas essayer de rendre les anciennes chaînes compatibles par magie.

## Avertissement important : le PoW mémoire a aussi un coût pour les honnêtes nœuds

Argon2id impose du temps et de la mémoire à **chaque tentative de minage**, mais aussi à **chaque vérification d'un bloc reçu**. Des paramètres trop élevés peuvent provoquer des ralentissements de synchronisation ou de saturation, notamment sur de petits PC et Raspberry Pi. Les paramètres d'un schéma de dérivation de mots de passe **ne s'importent pas tels quels** dans le consensus.

Ni SHA-256 ni Argon2id ne suppriment, à eux seuls, les attaques de majorité, les GPU/ASIC, l'éclipse de pairs ou le risque d'une petite chaîne faible en puissance de travail.

## Tests nécessaires avant de recommander un remplacement (NON EFFECTUÉS)

1. Benchmark comparable de SHA-256 et de trois configurations **candidates non activées** Argon2id sur Windows, Linux et appareil à mémoire limitée ; mesurer hashes/s, durée de validation, mémoire maximale, CPU, latence et variance.
2. Vérifier 1, 10, 100, 1 000, puis 10 000 blocs indépendamment ; mesurer synchro, RSS, latence et stockage, y compris sur branche concurrente ou après réorganisation.
3. Tester en adversarial : trames tronquées, blocs invalides en série, allocations excessives, exhaustion CPU/RAM et isolation de ressources. Protéger l'admission/anti-DoS.
4. Obtenir des **vecteurs déterministes sur deux implémentations ou environnements**, avec paramètres figés, erreurs et bornes ; comparer l'endianness, les limites et les formats de hash.
5. Définir difficulté, travail cumulé, fréquence cible, règles de version, génèse et politique de migration ; contre-revue, documentation et autorisation explicite avant tout changement R3.

### Points de départ du code

- `crates/aretovus-consensus/src/testnet.rs` : paramètres et fingerprint testnet actuel.
- `crates/aretovus-consensus/src/proof_of_work.rs` : helper PoW SHA-256 existant.
- `crates/aretovus-consensus/src/account_block.rs` : validation PoW.
- `crates/aretovus-consensus/src/reward.rs` : émission et trésorerie — **hors périmètre** de cette RFC.
- `crates/aretovus-node/src/p2p.rs` : admission réseau et synchronisation.
- `crates/aretovus-wallet/Cargo.toml` : dépendance Argon2 pour le portefeuille, **pas pour le PoW**.

## Choix non tranché

**Nous voulons garder SHA-256.** Mais cela ne décide pas encore si le futur testnet doit miner exclusivement en SHA-256, exclusivement via un PoW mémoire versionné ou via une composition évaluée des deux. Pour l'instant, **le consensus reste inchangé**. Le bon choix sera fondé sur mesures, menace réelle et coût pour les petits nœuds, pas sur l'idée que SHA-256 serait « cassé ».

**Sécurité :** NO-GO mainnet et fonds réels tant que les conditions du projet ne sont pas remplies.

Sources techniques externes : [RFC 9106 Argon2](https://www.rfc-editor.org/rfc/rfc9106.html) ; [Bitcoin Developer Guide — block chain et attaques de majorité](https://developer.bitcoin.org/devguide/block_chain.html).
