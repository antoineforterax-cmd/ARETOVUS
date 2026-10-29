# Profil ML-DSA natif expérimental du testnet

Le testnet courant utilise désormais le format transaction v3: l’autorisation est exclusivement une signature ML-DSA-65 conforme au format de clé et de signature FIPS 204. Le format v3 est la seule version de transaction acceptée par le consensus Testnet. Les formats Ed25519 v1 et hybride v2 restent disponibles dans les outils et tests Regtest, mais le testnet les rejette.

## Encodage v3

Le digest signé engage la version, le réseau, l’identifiant de l’expéditeur, le nonce, le destinataire, le montant et les frais. `auth_data` a une longueur fixe de 5 261 octets:

| Champ | Taille |
|---|---:|
| Clé publique ML-DSA-65 encodée | 1 952 octets |
| Signature ML-DSA-65 encodée | 3 309 octets |
| **Total** | **5 261 octets** |

L’identifiant de compte vaut `H(AURORA/exp/v0/post-quantum-account-id, clé_publique_ML-DSA-65)`. Il reste sur 32 octets. Cette valeur est un identifiant interne expérimental, pas encore une adresse utilisateur lisible ou un QR code. Il n’existe pas de migration des comptes v1/v2: les fonds et identifiants de l’ancien testnet ne se reportent pas au nouveau profil.

Le handshake pair-à-pair engage un identifiant de profil distinct. Les pairs de l’ancien testnet hybride sont donc incompatibles. Utiliser un nouveau `--data-dir` et conserver les anciens snapshots séparément; ne pas les effacer avant sauvegarde.

## Portée et limites

Le changement rend la signature des transactions du testnet post-quantique selon le profil ML-DSA-65. Il ne rend pas toute la blockchain « 100 % post-quantique »: le hachage des identifiants, l’engagement des blocs et le PoW restent fondés sur SHA-256; le réseau et ses dépendances n’ont pas fait l’objet d’une revue post-quantique globale.

La dépendance RustCrypto `ml-dsa` est épinglée à 0.1.1. L’utilisation d’un algorithme standardisé ne certifie ni l’implémentation, ni l’intégration, ni le générateur et le stockage de clés, ni la résistance complète aux canaux auxiliaires. Il faut encore exécuter des vecteurs normatifs indépendants et demander une revue cryptographique externe avant toute utilisation au-delà du testnet.

Chaque transaction transporte plus de 5 KiB d’autorisation. Cela réduit le nombre de transactions par bloc de 4 MiB et augmente les coûts CPU, mémoire, stockage et bande passante. La taille maximale des blocs et des frames réseau ne change pas; les essais de charge réels restent à faire.

## Fonctions déjà présentes

Le portefeuille CLI génère et utilise des clés ML-DSA-65. Il chiffre ses nouveaux fichiers par défaut et propose sauvegarde et restauration chiffrées. Les racines et preuves Merkle, le téléchargement et la vérification SPV d’en-têtes et la bibliothèque d’URI `aurora://` sont intégrés. Ces fonctions restent expérimentales; une preuve d’inclusion fournie par un pair ne garantit pas l’absence d’éclipse. Voir le [guide du portefeuille](guide-fr/05-portefeuille.md).

## Vérifications et travaux restants

1. Mesurer vérification, mémoire, propagation et taille de blocs avec des transactions v3 réelles sur des campagnes prolongées.
2. Valider l’implémentation contre les vecteurs FIPS 204, consigner le commit et le résultat CI, puis obtenir une revue cryptographique indépendante. La présence d’un test dans la branche ne prouve pas à elle seule son exécution réussie.
3. Éprouver les sauvegardes et restaurations de clés, les erreurs de mot de passe et le comportement sur les systèmes pris en charge.
4. Concevoir séparément DID, authentification challenge-response, gardiens et time-locks. Ces mécanismes changent l’autorisation et l’état des comptes et ne doivent pas être ajoutés sans règles d’activation, révocation, seuil, récupération et délai.

Ne pas utiliser avec de vrais fonds.
