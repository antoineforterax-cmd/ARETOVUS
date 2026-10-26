# Décisions du testnet expérimental AURORA

**Date :** 26 octobre 2026. **Portée :** branche `feature/emission-schedule-v0`, réseau de test supervisé. Ce document décrit les règles exécutées et les choix de périmètre; il ne valide ni mainnet ni usage de fonds réels.

## Règles économiques actives

- Genèse vide, sans préminage. Plafond brut strict : 500 000 000 AUR à 8 décimales.
- Par bloc, `R = min(reliquat, max(1, floor(reliquat / 1_000_000)))` unités minimales. Le premier bloc émet 500 AUR. Les réorganisations recalculent l’offre de chaque branche.
- Trésorerie : `floor(R / 20)` au compte fixe testnet `[0xA0; 32]`; le reste au mineur.
- Frais : `floor(F / 2)` au mineur et le reliquat brûlé. Le burn ne réouvre pas le plafond. Ce taux est fixe dans ce profil; un taux dynamique exigerait une nouvelle règle de consensus et des essais adversariaux.
- Après épuisement du plafond, le mineur ne reçoit plus d’émission. Sa part des frais reste 50 %.

## Trésorerie et propriété

Le compte de trésorerie reçoit les unités prévues, mais ce profil ne définit aucune dépense, clé d’administration, gouvernance ou contrat. Aucune personne ne doit présenter ces unités comme un budget disponible. Une future politique devra spécifier qui propose et autorise une dépense, ses preuves, la transparence, le contrôle des conflits d’intérêts et le traitement d’une réorganisation. Son activation exigerait une proposition de protocole, des tests et un réseau/version compatibles. Le compte fixe actuel ne garantit aucune voie de dépense future.

Aucun solde inactif n’est saisi, recyclé, expiré ou réattribué. Un tel changement toucherait aux droits de propriété et n’entre pas dans le profil expérimental.

## Identité et récupération

Les DID, défis signés, gardiens et délais sont décrits dans [DID_RECOVERY_DESIGN.md](DID_RECOVERY_DESIGN.md), sans champs de compte ni transaction active. La version v3 du testnet ne contient aucune récupération de fonds. Le futur profil envisagé requiert un identifiant de compte stable, des règles d’activation et une analyse spécifique des collusions, pertes de clés et réorganisations. Les détails de ce futur profil restent à décider avant son codage.

## Revue et portée de la poursuite

Le porteur a choisi le 26 octobre 2026 de **continuer le développement expérimental sans revue externe immédiate**. Le [dossier de revue](INDEPENDENT_REVIEW.md) reste disponible. Aucune revue indépendante n’a eu lieu; cette décision ne constitue pas une attestation de sécurité. Une revue humaine indépendante reste nécessaire avant toute prétention de sécurité mainnet ou d’usage de fonds réels.

Les tests CI, les campagnes de fuzzing et les simulations documentées servent à détecter des défauts, mais ne remplacent pas cette revue. Les applications futures doivent afficher ce statut expérimental.
