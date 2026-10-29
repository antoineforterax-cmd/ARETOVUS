# Protection volontaire des fonds — wallet testnet

Ce chantier met en œuvre la première étape de la section 7 du
[cahier des charges](planning/CAHIER_DES_CHARGES.md) : vérification des sauvegardes,
relecture des paiements avant signature et plafonds volontaires.

## Paiement relu avant signature

`aurora-wallet send` affiche le réseau AURORA Testnet, le compte source,
l'adresse complète du destinataire, le montant, les frais, le débit total,
le nœud RPC et le nonce. Par défaut, il faut saisir exactement
`SEND ADRESSE_COMPLETE`, puis Entrée. Une entrée vide, différente, tronquée ou
une fin d'entrée annule avant signature. Un montant nul et un paiement vers
le compte source sont refusés par ce parcours de paiement.

```sh
aurora-wallet send --key-file wallet.aurkey --to ADRESSE_HEX --amount 1 --fee 0.00000003 --node 127.0.0.1:7000 --dry-run
aurora-wallet send --key-file wallet.aurkey --to ADRESSE_HEX --amount 1 --fee 0.00000003 --node 127.0.0.1:7000 --max-amount 1 --max-fee 0.00000003
```

Remplacer l'adresse et le port par ceux du testnet utilisé. L'aperçu
`--dry-run` consulte le compte mais ne signe et ne soumet aucune transaction.
Les plafonds sont inclusifs, exprimés en AUR et vérifiés avec l'arithmétique
entière du protocole. Ils s'appliquent à cet appel uniquement ; ils ne sont
pas enregistrés comme politique permanente et ne sont pas imposés par le
consensus. Le solde et le nonce tiennent compte des transactions en attente
sur le nœud interrogé. Une nouvelle lecture juste avant signature refuse
l'envoi si le compte a changé pendant la relecture. Un changement ultérieur
reste possible ; le nœud doit toujours valider la transaction.

L'automatisation doit choisir explicitement `--yes` pour supprimer la saisie
interactive. Ce choix conserve les contrôles de solde, les plafonds fournis
et la seconde lecture du compte. Il ne peut pas être combiné à `--dry-run`.

## Vérifier une sauvegarde avant d'en avoir besoin

```sh
aurora-wallet backup --key-file wallet.aurkey --out backup.aurkey
aurora-wallet verify-backup --from backup.aurkey --expected-account ADRESSE_HEX
aurora-wallet restore --from backup.aurkey --out restored.aurkey
```

Le mot de passe est demandé dans le terminal, jamais comme argument.
`verify-backup` authentifie le fichier chiffré puis vérifie que la clé
restaurée en mémoire dérive exactement le compte attendu. Elle n'écrit aucun
fichier et n'affiche aucune clé privée. Un mot de passe incorrect, un fichier
altéré ou une autre identité provoquent un échec. Les copies chiffrées
utilisent déjà un sel et un nonce neufs et refusent d'écraser une destination.

Conserver la sauvegarde et le mot de passe séparément ; tester régulièrement
la sauvegarde avec l'identité attendue conservée par un autre moyen.

## État du cahier des charges et limites

- Réalisé : sauvegarde/restauration chiffrées, vérification de sauvegarde,
  récapitulatif avant signature, confirmation renforcée et plafonds par appel.
- À concevoir : délais volontaires avec annulation et politique persistante.
- À concevoir et valider au niveau protocole : récupération par gardiens,
  rotation des clés et identité stable, voir [le dossier de conception](DID_RECOVERY_DESIGN.md).

Les comptes v3 sont dérivés de la clé publique. La perte de toutes les copies
de la clé ou de leur mot de passe n'est pas récupérable avec ces seules
protections. Une adresse hexadécimale valide ne prouve pas l'identité de son
propriétaire. Ces contrôles locaux ne remboursent pas un paiement confirmé et
ne donnent aucun pouvoir de gel ou de récupération à un administrateur.
Le testnet reste expérimental et requiert des audits indépendants avant un
usage financier réel.

## Validation automatisée

Les tests Rust couvrent les limites exactes, le dépassement monétaire,
le solde insuffisant, les confirmations erronées ou tronquées et la
vérification d'une sauvegarde contre une mauvaise identité ou un mauvais
mot de passe. Le smoke test à deux processus vérifie aussi que les aperçus,
annulations et refus par plafonds ne changent ni solde ni nonce, puis qu'un
paiement explicitement autorisé est inclus et vérifié par SPV.
