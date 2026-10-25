# Profil de signature hybride post-quantique expérimental

Ce profil testnet facultatif ne constitue pas une décision de lancement. Le format v1 Ed25519 reste accepté. Le format v2 exige simultanément Ed25519 et ML-DSA-65 (FIPS 204) pour autoriser le même digest canonique. Le passage à v2 ne migre pas automatiquement les comptes v1.

## Encodage v2

`sender` est `H(AURORA/exp/v0/hybrid-account-id, ed25519_public_key || mldsa65_public_key)`. `auth_data` est une concaténation de longueur fixe :

| Champ | Taille |
|---|---:|
| Clé publique Ed25519 | 32 octets |
| Clé publique ML-DSA-65 (encodage FIPS 204) | 1 952 octets |
| Signature Ed25519 | 64 octets |
| Signature ML-DSA-65 | 3 309 octets |
| **Total** | **5 357 octets** |

Le digest signé engage version, réseau, expéditeur, nonce, destinataire, montant et frais. Les deux signatures sont nécessaires; le format ne peut pas dégrader silencieusement vers Ed25519 seul.

## Limites de sécurité

La dépendance RustCrypto `ml-dsa` est épinglée à 0.1.1. Sa documentation annonce explicitement qu’elle n’a pas fait l’objet d’un audit indépendant. Le fait d’utiliser l’algorithme standardisé ne certifie ni cette implémentation, ni l’intégration, ni la génération et le stockage des clés, ni la résistance aux canaux auxiliaires, ni le protocole dans son ensemble. Le profil de signature utilisé doit encore être confronté à des vecteurs normatifs indépendants.

Les clés et signatures sont volumineuses; leurs effets sur la taille des blocs, snapshots, CPU, mémoire et protections anti-DoS restent à mesurer. Avant toute adoption hors testnet, il faut une revue indépendante, des vecteurs FIPS 204, des essais d’interopérabilité, une stratégie de migration des comptes et des essais de gestion des clés. Ne pas utiliser avec de vrais fonds.
