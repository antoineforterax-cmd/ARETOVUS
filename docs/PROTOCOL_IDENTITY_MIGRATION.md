# Identité de protocole ARETOVUS — rupture expérimentale contrôlée

Cette branche expérimente un **nouveau réseau** : le nom de la blockchain, son espace cryptographique, ses identifiants de stockage, ses clés locales, ses adresses URI, les messages du portefeuille et la genèse sont désormais ARETOVUS.

## Orientations conservées — décision A3 du 1er novembre 2026

La décision [ARETOVUS #142](https://github.com/antoineforterax-cmd/ARETOVUS/issues/142#issuecomment-6557027057) est de **repartir sur un réseau natif indépendant**, tout en conservant les directions de développement et les protections acquises.

**Nous conservons les règles numériques du testnet historique sur `main` au commit `7eb3924e508e80307a221b23b322da62ff08d814` :** plafond expérimental de 500 000 000 unités (8 décimales), calcul d'émission `R = min(reliquat, max(1, floor(reliquat / 1_000_000)))`, trésorerie de `floor(R/20)` et partage des frais `floor(F/2)` au mineur, reliquat brûlé. La nouvelle **identité cryptographique** n'autorise aucun changement implicite d'émission, de difficulté, de frais, de propriété ou de règles de réorganisation. Le plafond final envisagé pour un éventuel mainnet, les symboles monétaires et leur disponibilité restent **des décisions distinctes, non prises ici**.

**Nous conservons tous les correctifs récents de `main`, en particulier la [PR #140](https://github.com/antoineforterax-cmd/ARETOVUS/pull/140) :** comparaison de l'identité des handles Windows, rejet des substitutions, contrôle `O_NOFOLLOW` sur Unix et protections de sauvegarde. Ils doivent être présents dans la version native et dans ses tests de régression. Les tests historiques, les logs et les anciens artefacts restent inchangés et archivés, mais ne sont pas importés dans le nouveau réseau.

**Nous conservons la trajectoire du projet :** fiabilité et audit adversarial avant nouveaux modules ; synchronisation longue et résistance P2P ; nœud praticable sur matériel modeste ; maîtrise de la dépense de stockage et de calcul ; décentralisation ; puis recherche sur confidentialité, récupération facultative et clients mobiles. Ces dernières fonctions **ne sont pas encore activées par cette migration**.

**Verrou de fusion :** les CI réussies sur l'ancienne base de la PR ne prouvent pas l'intégration avec `main`. Reconcilier explicitement les deux historiques, relancer Rust et Windows/Linux/macOS sur le SHA combiné, exiger une validation de fusion séparée ; **aucune publication ni argent réel**.

## Ruptures volontaires

- Les hachages de domaine utilisent désormais `ARETOVUS/exp/...` ; les identifiants de transactions et de comptes, les racines Merkle et les signatures en dépendent.
- La genèse de testnet (horodatage 1, difficulté initiale 8) est re-minée avec un nonce de **45** et le hash fixé à `002897e98fc5f76ea6b9a9e06eec414c6bcafd7171f73b644971a6b0aefbcfe3`.
- Les marqueurs de stockage à huit octets deviennent `ARETOSNP`, `ARETOIDX` et `ARETOPAG`; extensions `.aretidx`, `.aretblk`, `.aretpage`. Les magics de wallet sont `ARETOVK3` et `ARETOVK4`; extension `.aretkey`.
- Le portefeuille utilise `aretovus://testnet/`, `ARETOVUS-TESTNET`, la confirmation `SEND ARETOVUS-TESTNET`, et la récupération `aretovus-testnet-recovery-v1:`.
- L'unité affichée est **ARETOVUS**. Le plafond, les décimales et les règles d'émission restent les mêmes nombres; le changement porte sur l'identité du protocole.

## Sécurité des anciennes données

Cette branche **ne migre pas** les anciens fichiers : les archives, clés et blocs des expérimentations précédentes ne doivent jamais être écrasés. Les utilisateurs conservent les anciens binaires et les copies des anciens répertoires pour leur propre archivage, sans lancer cette version dessus.

Le nœud refuse d'initialiser une genèse dans un répertoire **non vide sans snapshot ARETOVUS reconnu**. Il faut utiliser un **nouveau dossier de données** et de nouvelles clés de test. Les anciennes transactions et signatures ne peuvent pas être transférées par un simple renommage ; la chaîne ARETOVUS ne prétend pas reprendre les soldes précédents.

## Validation obligatoire

1. Exécuter les vecteurs Python indépendants et les tests Rust de hachage et de genèse.
2. Vérifier l'isolation réseau : mauvais fingerprint, mauvais schéma URI, mauvaise signature, anciens formats de stockage/clé.
3. Rejouer les tests de persistance et restauration, corruption, crash, disque plein, P2P, partitions et transactions.
4. Vérifier les archives Windows, Linux, macOS et la synchronisation entre deux nouvelles instances.
5. Vérifier `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace --locked`, `cargo test --workspace --locked`, `python3 scripts/testnet_smoke.py`.

**Statut :** branche expérimentale, pas une migration en production ni une validation de mainnet. Une CI verte ne remplace pas une revue indépendante.
