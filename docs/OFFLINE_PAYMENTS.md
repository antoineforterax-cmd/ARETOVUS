# Signer hors ligne, diffuser sans clé privée

Ce chantier réalise le parcours logiciel de paiements préparés hors ligne
du cahier des charges (§9.4), en conservant les contrôles volontaires du
wallet (§7) et sans nouvelle règle de consensus.

## Préparation

Sur un nœud connecté, relever le compte source et son prochain nonce. Le
wallet `balance` fournit un solde et un nonce ajustés des transactions en
attente sur ce nœud, sans constituer une preuve de l'état confirmé. Apporter
ces informations sur la machine qui conserve la clé. Vérifier le destinataire
et le montant par un canal de confiance.

## Machine qui conserve la clé, sans connexion

```sh
aurora-wallet sign-offline --key-file wallet.aurkey --to ADRESSE_HEX --amount 1 --fee 0.00000003 --nonce 0 --out payment.aurtx
```

Remplacer les valeurs, notamment le nonce, par celles du paiement prévu.
Le mot de passe du wallet chiffré est demandé normalement. Cette commande
ne fait aucun appel réseau et refuse `--node`. Elle affiche source,
destinataire, montant, frais, débit et nonce avant signature. Le solde est
explicitement marqué non vérifié et le nonce comme fourni manuellement.

La confirmation exige la saisie exacte `SEND ADRESSE_COMPLETE`, puis Entrée.
Une annulation ne signe et n'écrit rien. `--dry-run` affiche un aperçu sans
signature ni écriture ; `--yes` choisit explicitement l'automatisation.
Ces deux options ne se combinent pas.

La politique `wallet.aurkey.limits` est chargée automatiquement. Les
plafonds ponctuels `--max-amount` et `--max-fee` ne peuvent que la
renforcer. Elle est relue après confirmation avant signature ; un changement
annule l'opération. Un nonce égal à u64::MAX est refusé par ce parcours, car
il ne permet plus de progression du compteur.

Le fichier est créé sans écrasement, avec permissions 0600 sous Unix
(droits hérités du dossier sous Windows). Il contient uniquement la
transaction publique v3 signée : aucun mot de passe, code de récupération
ni graine privée. Transférer ce fichier à la machine connectée.

## Machine connectée, sans clé privée

```sh
aurora-wallet broadcast --from payment.aurtx --node 127.0.0.1:7000
aurora-wallet status --node 127.0.0.1:7000 --txid ID_TRANSACTION
aurora-wallet verify --node 127.0.0.1:7000 --txid ID_TRANSACTION --confirmations 1
```

Adapter le port et le nombre de confirmations. La diffusion charge le
fichier une seule fois, vérifie l'encodage canonique, le profil testnet v3,
les champs et la signature ML-DSA-65, puis affiche le paiement et son txid.
Elle demande de nouveau la confirmation du destinataire, sauf avec
`--yes`. Elle ne charge aucune clé privée et refuse les options de
modification des champs signés et les options répétées.

Après confirmation, elle contrôle le nonce exact et le solde sur le nœud,
puis transmet les mêmes octets canoniques de transaction. Le nœud garde
l'autorité de validation ; un changement d'état entre la lecture et la
soumission peut provoquer un rejet. Le txid acquitté doit correspondre à
celui du fichier. Une acceptation en mempool reste une attente de minage.
Consulter le statut et les preuves d'inclusion pour vérifier la suite.

Un fichier déjà accepté ou confirmé ne doit pas être resigné aveuglément
avec un autre nonce. En cas de panne réseau ou d'acquittement incohérent,
le paiement peut avoir été transmis : vérifier le txid préparé avant de
créer une autre autorisation. Le refus d'un nonce déjà consommé ne prouve
pas à lui seul la confirmation de cette transaction.

## Menaces et limites

- Le fichier signé est une **autorisation de dépense**, pas un brouillon.
  Tout détenteur peut le diffuser. Sa suppression, un refus de diffusion
  ou une modification ultérieure des plafonds ne révoquent pas les copies.
- Les transactions v3 n'ont pas d'expiration. Un fichier peut rester
  utilisable tant que son nonce et les règles d'état permettent l'exécution.
- Hors ligne, le wallet ne vérifie ni solde courant, ni nonce courant, ni
  disponibilité du nœud. Il ne réserve pas les fonds. Plusieurs fichiers
  utilisant le même nonce ne permettent pas plusieurs dépenses valides.
- La vérification de signature détecte les modifications du paiement,
  sans certifier l'identité civile du destinataire ni la fiabilité de la
  machine de signature. Une machine compromise peut tromper l'utilisateur.
- La transaction conserve les métadonnées publiques du testnet. Le parcours
  n'apporte pas de confidentialité native des montants ou des adresses.
- Les plafonds sont appliqués à la signature sur le wallet local. Le
  diffuseur sans clé n'a pas cette configuration et ne l'applique pas
  rétroactivement à un fichier déjà signé.

## Format, ressources et compatibilité

Conteneur `AURTX3\0\0` de 8 octets suivi de l'encodage canonique existant
de AccountTransaction. Le lecteur refuse les fichiers non réguliers, les
fichiers dépassant 8 Kio, les troncatures et les octets supplémentaires.
Il ne décode et ne vérifie qu'une transaction par fichier. La signature
authentifie les champs ; aucune primitive cryptographique ni dépendance
supplémentaire n'est ajoutée.

Seul le profil ML-DSA-65 testnet v3 est accepté. Le réseau et les champs
sont ceux déjà engagés par sa signature ; aucune liaison supplémentaire
à une instance de chaîne n'est inventée par le conteneur. Le protocole
réseau, les règles monétaires et le stockage du nœud sont inchangés.
Les anciens wallets ne disposent pas de ces commandes ; revenir à une
ancienne version n'annule aucun fichier signé existant.

## Validation

Tests Rust : conservation exacte des champs et du txid, permissions,
refus d'écrasement, signatures et champs altérés, mauvais réseau/version,
troncature, dépassement de taille et données supplémentaires.

Test CLI avec nœuds : annulation et aperçu sans création de fichier,
refus des options inconnues/répétées et des plafonds dépassés, signature
sans paramètre de nœud et sans changement du solde/nonce. Le fichier est
ensuite diffusé pendant que le fichier de clé est indisponible ; le txid
reste identique et le paiement est confirmé par minage. Les diffusions
altérées, non confirmées ou avec nonce consommé sont refusées.

Ces tests utilisent plusieurs processus sur une machine. Les essais
physiques entre deux ordinateurs, l'exploitation réellement déconnectée,
la compatibilité avec du matériel dédié et un audit indépendant restent
à valider. Le testnet reste expérimental.
