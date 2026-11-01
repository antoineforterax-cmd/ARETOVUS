# 1. Comprendre ARETOVUS

[Sommaire](README.md) · [Économie →](02-economie.md)

## Le but du projet

ARETOVUS explore une monnaie numérique dont les règles peuvent être vérifiées et exécutées localement. Le projet associe une émission limitée et dégressive, des paiements signés avec ML-DSA-65, et des nœuds qui conservent et valident eux-mêmes leur chaîne. L’objectif est de rendre le fonctionnement observable et reproductible sur des ordinateurs individuels, puis de mesurer ses limites avant d’en élargir l’usage.

Ce but décrit le travail demandé et le logiciel en construction. Il ne prédit ni adoption, ni valeur marchande, ni rentabilité du minage. Il n’existe pas aujourd’hui de réseau public déclaré stable, de cotation officielle ni de mécanisme de vente livré par ce projet.

## Ce que l’on télécharge

L’archive fournit deux programmes :

- **`aretovus-node`** vérifie et conserve les blocs, communique avec les pairs, sert les requêtes du portefeuille et peut miner des blocs de test.
- **`aretovus-wallet`** crée une clé, dérive son adresse, signe des paiements et interroge un nœud. Il fonctionne en ligne de commande.

L’archive ne contient pas une blockchain complète à jour. Le nœud crée la genèse déterministe de ce testnet, puis reçoit des blocs de ses pairs ou en mine. Chaque bloc reçu est vérifié avant d’être accepté. Le dossier choisi avec `--data-dir` contient la copie locale de la chaîne.

Le nœud n’a pas besoin d’un compte auprès d’un fournisseur d’API pour valider les blocs. GitHub sert ici à distribuer le code et les binaires et à exécuter les contrôles du projet; ce n’est pas un serveur de consensus nécessaire au fonctionnement d’un réseau local déjà lancé.

## Les mots utiles

| Mot | Sens dans ARETOVUS |
| --- | --- |
| AUR | Unité monétaire du protocole; un AUR est divisé en 100 000 000 unités minimales. |
| Adresse | Identifiant public du compte, affiché actuellement comme 64 caractères hexadécimaux. Ce n’est pas la clé privée. |
| Clé privée | Secret permettant de signer. Le fichier de portefeuille la protège par chiffrement lorsqu’il est créé sans l’option de test en clair. |
| Transaction | Ordre signé de transférer un montant, avec des frais et un numéro de séquence. |
| Nonce de compte | Numéro attendu pour le prochain transfert, utilisé notamment contre le rejeu. Le wallet le demande au nœud. |
| Mempool | Ensemble local de transactions acceptées en attente d’un bloc. Il reste en mémoire et peut être perdu au redémarrage. |
| Bloc | Ensemble ordonné de transactions rattaché à un parent et soumis aux règles de preuve de travail. |
| Hauteur | Position d’un bloc après la genèse. Deux branches peuvent avoir la même hauteur. |
| Sommet, ou `tip` | Identifiant du dernier bloc de la chaîne retenue par le nœud. Comparer hauteur **et** sommet pour vérifier une convergence. |
| Pair | Autre nœud auquel on se connecte pour échanger des données. |
| Mineur | Processus qui cherche une preuve de travail valide et propose un bloc. |
| Réorganisation | Changement de branche retenue lorsqu’une autre chaîne valide est préférée par la règle de travail cumulé. |

## D’un paiement à un bloc

Le wallet construit et signe un transfert, puis l’envoie à un nœud. Celui-ci vérifie notamment la signature, le nonce, le solde et les limites du protocole. Si le transfert est accepté, il entre dans le mempool. Un mineur le récupère, construit un bloc et cherche une preuve de travail. Les autres nœuds revalident ce bloc et mettent à jour leur état s’ils l’acceptent.

Une inclusion peut disparaître lors d’une réorganisation. Le statut `Confirmed` du wallet décrit la chaîne du nœud interrogé à cet instant; ce n’est pas une promesse d’irréversibilité.

## Ce qui est disponible aujourd’hui

Les nœuds locaux, le minage de test, le portefeuille CLI chiffré, les transactions natives ML-DSA-65, le stockage persistant, les réorganisations, les preuves Merkle et la vérification légère SPV sont présents. La découverte des pairs est limitée et part d’adresses connues. Les interfaces graphiques, l’explorateur, le site de présentation, les fonctions de vente et le mobile sont reportés.

La récupération par gardiens, les DID dans les comptes, la gouvernance de trésorerie et le recyclage des soldes inactifs ne font pas partie des règles actives. La [dernière page](07-technologie-et-suite.md) distingue les propositions et les fonctions déjà livrées.
