# Récupérer le wallet après perte du fichier ou du mot de passe

Le wallet v3 testnet peut désormais exporter un **code de secours hors ligne**
qui régénère la même clé ML-DSA-65 et le même compte. Le fichier wallet
d'origine et son ancien mot de passe ne sont pas nécessaires pour restaurer.
Les fonds restent sur le même compte de la chaîne.

Le code doit être créé et conservé **avant** la perte. Il ne permet pas de
récupérer un compte pour lequel aucun secret de secours n'a été conservé.

## Préparer et vérifier le secours

```sh
aretovus-wallet recovery-export --key-file wallet.aretkey --out recovery-secret.txt --acknowledge-secret
aretovus-wallet recovery-verify --from recovery-secret.txt --expected-account ADRESSE_HEX
```

Le mot de passe actuel est demandé si le wallet est chiffré. Le code
n'apparaît pas dans la sortie de la commande. Le fichier est créé sans
écrasement ; sous Unix ses permissions sont 0600. Sous Windows, les droits
hérités du dossier restent à gérer par l'utilisateur.

Ce fichier contient un **secret non chiffré donnant accès à tous les fonds**.
On peut en garder une copie papier dans un lieu protégé, séparé du wallet.
Éviter les captures d'écran, la messagerie, les dépôts Git, les journaux et les
imprimantes partagées. Vérifier la copie réellement conservée : pour un code
papier, lancer la vérification sans `--from` et saisir le code dans l'invite
masquée. Conserver aussi l'adresse publique attendue par un moyen indépendant.

Le format est spécifique à ARETOVUS testnet v1 : préfixe de réseau/version,
graine de 32 octets en hexadécimal, puis somme de contrôle SHA-256 complète.
La somme de contrôle détecte les fautes de copie ; elle ne chiffre pas le
secret et ne remplace pas la vérification de l'adresse attendue. Il ne s'agit
pas d'une phrase BIP39 et ce code n'est pas un format universel de wallet.

## Restaurer après la perte

```sh
aretovus-wallet recovery-restore --out recovered.aretkey --expected-account ADRESSE_HEX
```

Saisir le code de secours dans l'invite masquée, puis choisir et confirmer
un **nouveau mot de passe** pour le nouveau wallet chiffré. L'ancien mot de
passe n'est pas demandé. Pour un secours déjà disponible sous forme de
fichier, ajouter `--from recovery-secret.txt`.

L'opération est hors ligne : aucun nœud, service central ou gardien n'est
contacté. Une erreur de format, de contrôle ou de compte attendu bloque la
restauration. Une destination existante n'est jamais écrasée. Les nouvelles
copies chiffrées utilisent le format ARETOVK4 existant, avec sel et nonce neufs.
Le lecteur de fichier limite les données à 257 octets pour refuser les entrées
trop grandes. Les nouveaux tampons contenant le secours sont effacés à leur
libération en mémoire ; cela ne garantit pas l'effacement du disque, du
presse-papiers, des sauvegardes du système ou de toute copie externe.

`--plain-testnet` existe pour les tests automatisés et restaure une clé non
chiffrée ; ne pas l'utiliser comme parcours normal de restauration.

## Ce qui est restauré, et ce qui reste à développer

Le code rétablit la clé d'origine : il **ne la révoque pas**. Si un voleur
possède cette clé ou le code, il garde le même pouvoir de dépense. Une clé
soupçonnée compromise nécessite un transfert vers un nouveau compte sûr.

Cette solution couvre la perte du fichier wallet et de son mot de passe,
tant que le code est conservé. Elle ne remplace pas la récupération sociale
avec rotation de clé : [ce profil de consensus reste à concevoir et revoir](DID_RECOVERY_DESIGN.md).

Si la clé, les sauvegardes et tous les moyens de secours sont perdus,
aucune information cryptographique ne permet d'identifier le propriétaire
légitime. Aucun administrateur n'a de clé universelle de récupération.

## Tests

- Export, suppression du wallet chiffré d'origine, restauration chiffrée avec
  un autre mot de passe, puis signature ML-DSA-65 vérifiée.
- Refus des corruptions caractère par caractère, des troncatures, des
  caractères non ASCII, des identités différentes et des fichiers trop grands.
- Refus d'écrasement et contrôle des permissions Unix du secours.
- Test avec un compte déjà financé sur un nœud : suppression du wallet,
  restauration via les commandes CLI, solde/nonce inchangés, puis paiement
  signé avec la clé restaurée et confirmé par minage.

Ces essais concernent le testnet expérimental ; ils ne constituent pas un
audit indépendant du dispositif de sauvegarde.
