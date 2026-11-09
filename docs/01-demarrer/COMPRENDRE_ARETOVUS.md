# Comprendre ARETOVUS — de zéro au fonctionnement technique

Ce guide s'adresse aux curieux et aux personnes qui découvrent la programmation. Il décrit le **testnet expérimental** au niveau conceptuel ; les détails exécutables sont dans le [protocole](../PROTOCOL.md). Aucune unité de test ne doit être considérée comme de l'argent réel.

## 1. Qu'est-ce qu'une blockchain ?

Une blockchain est un registre de données partagé entre plusieurs ordinateurs, appelés **nœuds**. Chacun vérifie que les nouveaux blocs suivent les règles du protocole. Un bloc contient des transactions et une référence cryptographique au bloc précédent. Les nœuds n'acceptent pas un bloc simplement parce qu'un autre ordinateur le leur envoie.

ARETOVUS utilise un **modèle de comptes** : chaque adresse a un solde et un numéro de transaction, appelé *nonce*. Cela évite notamment qu'une transaction ancienne puisse être rejouée comme un nouveau paiement.

## 2. Les composants

| Composant | Image simple | Rôle |
| --- | --- | --- |
| **Wallet (portefeuille)** | Le trousseau de clés | Crée et protège les clés, prépare et signe les paiements |
| **Nœud** | Le vérificateur | Vérifie les blocs, garde l'état de la chaîne et dialogue avec des pairs |
| **Mempool** | La file d'attente | Conserve temporairement les transactions valides non encore incluses dans un bloc |
| **Mineur** | Le constructeur de blocs | Cherche un bloc dont le hash satisfait à la difficulté de preuve de travail |
| **Consensus** | Le règlement | Décide quels blocs et transactions sont valides et quelle branche devient canonique |
| **Stockage** | Les archives locales | Conserve blocs et index, vérifie les intégrités, recharge les données après redémarrage |
| **Explorateur** | La fenêtre de lecture | Affiche des informations sur la chaîne, sans être lui-même source de vérité |

Les cinq crates principales sont détaillées dans [ARCHITECTURE.md](../../ARCHITECTURE.md). L'application Tauri/React reste un prototype de **consultation**.

## 3. Que se passe-t-il quand Alice envoie à Bob ?

1. Alice crée un **portefeuille de test** et obtient une adresse. **Sa clé privée ne doit jamais être publiée.**
2. Le wallet interroge le nœud pour connaître l'état disponible, notamment le **nonce** de son compte.
3. Il construit une transaction : réseau, émetteur, destinataire, montant, frais et nonce.
4. Il signe cette transaction avec la clé privée (profil testnet **ML-DSA-65**). **Le nœud reçoit la transaction signée, pas la clé privée.**
5. Le nœud vérifie les formats, la signature, le solde, les frais et le nonce avant admission à son **mempool**. Une transaction refusée n'est pas un paiement confirmé.
6. Le mineur choisit des transactions et construit un bloc. Les identifiants de transactions sont engagés dans une **racine de Merkle**.
7. Le mineur essaie des valeurs de nonce de bloc jusqu'à obtenir un **SHA-256** satisfaisant à la difficulté.
8. Le bloc est diffusé aux pairs connus. Chaque nœud **revérifie** lui-même le bloc, l'émission, les transactions et son lien au parent.
9. Si le bloc appartient à la branche sélectionnée, les soldes et les nonces sont mis à jour. Une réorganisation de chaîne peut encore changer le résultat, d'où l'importance de l'état de confirmation.

### Schéma simplifié

```text
[Wallet Alice] -- transaction signée --> [Nœud A / mempool]
                                           |
                                           v
                                   [Mineur / PoW SHA-256]
                                           |
                                        [Bloc]
                                           |
                              diffusion vers pairs TCP/P2P
                                           |
                          [Nœud B vérifie indépendamment]
                                           |
                                [Chaîne et état des comptes]
                                           |
                            [Explorateur lit les données]
```

Ce schéma représente les rôles, pas une promesse d'un réseau public déjà décentralisé.

## 4. D'où viennent les unités du testnet ?

Le testnet démarre avec une genèse **sans préminage**. Les récompenses proviennent du calcul de l'émission prévue par le protocole lorsqu'un bloc est validé. Le plafond brut est **500 millions ARETOVUS**, avec huit décimales. La première émission brute est de **500 ARETOVUS** ; ensuite elle décroît avec le reliquat.

**Répartition de chaque émission brute :** 5 % arrondis à l'unité minimale inférieure pour le compte de trésorerie fixe, le reste au mineur. **Frais de transaction :** la moitié entière va au mineur et le reliquat est brûlé. Le burn ne réouvre pas le plafond d'émission. Le compte de trésorerie n'a **aucune procédure de dépense opérationnelle**.

Ces chiffres **ne préjugent pas d'une valeur marchande** et peuvent changer dans un **futur protocole distinct**, après décision explicite et tests.

## 5. Le minage est-il une vraie preuve de travail ?

Oui, le prototype calcule un PoW SHA-256. Mais ses paramètres sont destinés à un **testnet**, notamment une difficulté faible. Cela ne prouve ni rentabilité, ni résistance à un mineur dominant, ni décentralisation du réseau. Le minage de blocs de test n'est **pas** une activité financière.

## 6. Et si deux mineurs trouvent des blocs différents ?

Une **fourche (fork)** peut apparaître : deux branches contiennent des blocs concurrents. Les nœuds valident chaque branche et sélectionnent celle ayant le **travail cumulé** le plus élevé ; une règle de départage déterministe traite l'égalité. Une **réorganisation** peut retirer des blocs auparavant canoniques : transactions, soldes et récompenses doivent alors être recalculés correctement.

## 7. Comment un nœud conserve-t-il ses données ?

ARETOVUS écrit des blocs immuables et des index/manifestes contrôlés par sommes de vérification. Au redémarrage, le nœud vérifie les références et rejoue des blocs. Cela réduit certains risques de corruption, mais **ne prouve pas** l'absence de perte sur panne électrique, disque plein ou machines défaillantes.

## 8. Peut-on l'essayer sans savoir programmer ?

Oui, en local, sur un ordinateur compatible, avec **des données jetables et un réseau isolé**. Commencez par :

1. Lire la [page d'état](ETAT_ACTUEL.md) et le [guide français](../guide-fr/README.md).
2. Suivre l'[installation](../guide-fr/03-installation.md) ; ce guide utilise une **release datée d'octobre 2026**, donc vérifier le commit et la version.
3. Faire un [premier réseau local et du minage de test](../guide-fr/04-reseau-et-minage.md), puis un [paiement de test](../guide-fr/05-portefeuille.md).
4. Examiner l'[exploitation et les sauvegardes](../guide-fr/06-exploitation.md). Ne jamais exposer un RPC ni des clés réelles pour cette expérience.

Les tests automatisés sont décrits dans [CONTRIBUTING.md](../../CONTRIBUTING.md). Vous pouvez également participer en corrigeant la documentation, en posant une question technique ou en proposant un **test reproductible**, même sans maîtriser Rust.

## 9. Ce qu'ARETOVUS n'est pas encore

Ce n'est pas Ethereum : **pas d'environnement de contrats intelligents généralistes**, pas de DeFi intégrée. Ce n'est pas non plus un réseau anonyme prêt à l'emploi : Tor/I2P, montants cachés et confidentialité forte sont des **pistes de recherche**. Les signatures ML-DSA-65 ne rendent pas à elles seules tout le système « 100 % post-quantique ».

**Question suivante :** [Comment contribuer avec ou sans IA ?](../02-participer/CONTRIBUER_AVEC_IA.md) — ou consulter le [glossaire ci-dessous](#10-petit-glossaire).

## 10. Petit glossaire

- **Adresse :** identifiant public du destinataire.
- **Clé privée :** secret qui permet de signer ; ne jamais partager.
- **Signature :** preuve cryptographique qu'une transaction a été autorisée par la clé requise.
- **Hash :** empreinte numérique d'un contenu.
- **PoW :** preuve de travail exigée pour rendre un bloc admissible.
- **Nonce de compte :** compteur des transactions autorisées, distinct du nonce de minage d'un bloc.
- **Pair / P2P :** autre nœud avec lequel le logiciel échange.
- **Testnet / Regtest :** environnements d'essai, sans garantie de valeur monétaire.
- **Mainnet :** réseau public à vocation réelle ; **ARETOVUS n'en propose pas ici**.
