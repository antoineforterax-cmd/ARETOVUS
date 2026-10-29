# Plafonds volontaires persistants du wallet testnet

Ce chantier prolonge les limites volontaires de la section 7 du
[cahier des charges](planning/CAHIER_DES_CHARGES.md). Les plafonds par appel
existaient ; ils peuvent maintenant être enregistrés pour un fichier wallet
et appliqués automatiquement aux prochains paiements de ce wallet.

## Usage

```sh
aurora-wallet limits-create --key-file wallet.aurkey --max-amount 10 --max-fee 0.001
aurora-wallet limits-show --key-file wallet.aurkey
aurora-wallet send --key-file wallet.aurkey --to ADRESSE_HEX --amount 1 --fee 0.00000003 --node 127.0.0.1:7000
```

La création demande le mot de passe si le wallet est chiffré. Elle crée
`wallet.aurkey.limits`, sans écraser un fichier existant. Les deux plafonds
sont obligatoires, exprimés en AUR, inclusifs et par paiement. Le plafond de
montant doit être positif ; celui des frais peut être nul.

`send` charge automatiquement ce fichier, même avec `--yes` ou
`--dry-run`. Les options ponctuelles `--max-amount` et `--max-fee`
retiennent la valeur la plus basse entre la politique enregistrée et l'appel.
Les limites effectives sont affichées dans le récapitulatif. Les plafonds ne
sont pas des quotas journaliers : plusieurs paiements successifs peuvent
chacun atteindre le plafond.

Le fichier utilise un en-tête de version et de réseau fixe, le compte
attendu et deux montants. Une erreur de format, un compte différent, un
fichier illisible, trop grand, un lien symbolique ou un dossier bloquent
l'envoi. Seule l'absence du fichier signifie qu'aucune politique persistante
n'est configurée. Le wallet relit la politique après confirmation et annule
si elle a changé, avant de vérifier à nouveau le compte et de signer.

## Portée et contrôle de l'utilisateur

Cette protection est locale et facultative. Elle prévient les erreurs
d'usage du CLI ; elle ne résiste pas à une machine compromise ou à un acteur
capable de modifier la configuration, copier la clé ou utiliser un autre
logiciel de signature. Le fichier n'est pas signé et ne représente pas une
autorisation de consensus. Les dépenses restent gouvernées par la clé.

Il n'y a pas de quota global entre plusieurs copies du wallet. Le lien est
le chemin du fichier wallet : renommer ou restaurer le wallet sous un autre
nom exige de recopier son fichier `.limits` à côté du nouveau fichier.
La récupération de la clé ne restaure pas automatiquement cette configuration.
Sauvegarder la politique séparément du secret de récupération.

Pour ajuster les plafonds, modifier volontairement les deux montants dans
le fichier local, puis vérifier avec `limits-show`. Pour désactiver la
politique, retirer ce fichier. Une modification concurrente pendant un envoi
n'est pas un verrou interprocessus : la seconde lecture détecte les
changements jusqu'à ce point, sans garantir l'absence de changement ultérieur.
Aucune garantie de gel, délai de transfert ou annulation après signature
n'est ajoutée.

Le fichier contient l'adresse et les limites, aucune clé ni historique.
Ses permissions sont 0600 à la création sous Unix ; sous Windows, elles
dépendent du dossier parent. Aucun service distant ou télémétrie n'est ajouté.
L'absence de nouvelle dépendance ou de règle de consensus conserve les formats
v3 et AURKEY4. Revenir à une ancienne version du wallet supprime l'application
automatique de ces limites ; ne pas considérer ce retour arrière comme une
protection équivalente.

## Vérification

Les tests couvrent la composition des limites, leur rechargement, le refus
d'écrasement, la liaison au compte, les montants invalides, les champs inconnus
ou supplémentaires, les fichiers trop grands et les liens symboliques.
Le smoke test crée une politique puis lance de nouveaux processus CLI :
les plafonds restent appliqués malgré `--yes` et des options ponctuelles
plus élevées. Un fichier malformé bloque aussi l'envoi. Les refus ne changent
ni solde ni nonce ; un paiement aux bornes exactes est confirmé par minage.

Les délais annulables et les notifications locales restent des chantiers
distincts du cahier des charges.
