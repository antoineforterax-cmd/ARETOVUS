# Dossier de revue indépendante — AURORA

**Statut :** dossier préparatoire; aucune revue indépendante n’a été réalisée. Le porteur a choisi le 26 octobre 2026 de poursuivre le développement expérimental sans reviewer externe immédiat. Cette décision ne vaut pas revue et ne permet pas de revendiquer une sécurité pour des fonds réels.  
**Branche concernée :** `feature/emission-schedule-v0`.  
**Usage autorisé du code :** testnet local/expérimental uniquement; pas de fonds réels.

Une [contre-revue par un second agent IA](AI_COUNTER_REVIEW.md) a signalé quatre risques et ouvert les tâches de correction. Elle reste une aide interne; elle ne remplit pas le critère de revue humaine indépendante.

## Portée demandée

La revue devrait couvrir le code et le protocole au commit exact enregistré par l’auditeur :

1. Encodage canonique, identifiants de transaction et domaine de hachage.
2. Vérification ML-DSA-65, construction/signature de transaction, nonces, soldes et cas de rejeu.
3. Validation des blocs, engagements Merkle, limites de taille et preuve de travail SHA-256.
4. Retarget, règles d’horodatage/MTP, travail cumulé, égalités de travail et réorganisations.
5. Émission, plafond brut, trésorerie 5 %, frais 50/50, burns et état lors d’une réorganisation.
6. Décodage P2P/RPC, quotas, délais, synchronisation, découverte et gestion de données hostiles.
7. Stockage v5, lecture des snapshots v4, checksums, rejeu de toutes les branches, reprise, verrou exclusif de dossier entre processus et erreurs d’E/S.
8. Wallet testnet, fichiers de clés AURKEY4 chiffrés par défaut (Argon2id et XChaCha20-Poly1305), sauvegarde/restauration, format clair explicite réservé aux tests, URI et vérification SPV contre éclipses et données de pairs mensongères.

## Reproduction locale

Sur Linux avec Rust stable, Python 3 et un compilateur C++ pour libFuzzer :

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --locked
cargo test --workspace --locked
python3 scripts/testnet_smoke.py
python3 scripts/testnet_soak.py
cargo +nightly fuzz run canonical_decode -- -max_total_time=300
cargo +nightly fuzz run merkle_proofs -- -max_total_time=300
cargo +nightly fuzz run network_payloads -- -max_total_time=300
```

Les contrôles CI couvrent les tests unitaires, un scénario multi-processus et des campagnes fuzz courtes. Ils ne constituent pas une revue de sécurité. L’auditeur devrait relancer des campagnes plus longues, produire un rapport de menaces, examiner les diffs au commit épinglé et publier les constats avec des cas reproductibles.

## Limites connues à examiner

- Les paramètres de difficulté du testnet sont volontairement faibles et ne protègent pas des fonds réels. Le consensus exige des timestamps strictement croissants et un MTP, sans plafond lié à l’horloge locale; examiner les effets d’horodatages très futurs sur le retarget et la disponibilité des mineurs.
- Le nœud utilise une topologie amorcée par des adresses connues avec rattrapage et relais périodique des transactions depuis les pairs configurés; annonce spontanée à des pairs non configurés et découverte multi-sauts ne sont pas fournies. Les huit workers et la durée maximale de session de quinze minutes laissent un risque de saturation par connexions répétées ou entretenues; les tests de connexions lentes ne prouvent pas la résistance à cette saturation.
- Le portefeuille chiffre ses nouvelles clés par défaut, mais l’ergonomie du mot de passe, la dérivation, la sérialisation du fichier et la sauvegarde/restauration n’ont pas été revues indépendamment. Le format en clair est réservé aux scripts de test; ne jamais utiliser ces clés hors testnet.
- La preuve SPV dépend des pairs consultés et ne défend pas contre une éclipse.
- La trésorerie reçoit des unités à une adresse fixe, mais aucune gouvernance ni règle de dépense n’est définie.
- La chaîne conserve des états de comptes par bloc et clone l’état du parent à l’application d’un nouveau bloc; mesurer la mémoire et le temps de rejeu avec un grand nombre de comptes et de branches avant un testnet prolongé.
- Les tests de snapshot ne simulent pas une panne électrique réelle, le disque plein, ni une restauration interversion.
- Le SHA-256 utilisé pour les identifiants et le PoW n’est pas une construction résistante aux ordinateurs quantiques.

## Voie pratique sans engager d’audit payant

Le dépôt appartient à un compte personnel GitHub. Selon la [documentation GitHub sur les collaborateurs d’un dépôt personnel](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/repository-access-and-collaboration/inviting-collaborators-to-a-personal-repository), GitHub Free permet d’inviter des collaborateurs dans un dépôt privé. Une personne compétente et extérieure au développement peut donc recevoir l’accès au dépôt, examiner cette PR et remettre ses constats sans qu’une société d’audit soit nécessairement engagée. Cette voie dépend de sa disponibilité et ne garantit ni une revue gratuite ni sa qualité.

Étapes quand le porteur aura choisi cette personne : lui communiquer le périmètre ci-dessus, épingler le commit à examiner, lui donner accès au dépôt privé par les réglages GitHub, puis [demander sa revue de la PR](https://docs.github.com/en/pull-requests/how-tos/create-pull-requests/requesting-a-pull-request-review). Conserver les constats écrits, corriger les défauts et demander la vérification des correctifs. Aucun accès ni demande de revue n’a été accordé à ce jour. Une auto-revue, un second outil automatique ou une autre session d’IA peuvent aider à trouver des défauts, mais ne remplissent pas cette étape humaine indépendante.

## Résultat de revue

À compléter par la personne indépendante :

- Auditeur, affiliation et conflits d’intérêts :
- Commit exact examiné :
- Périmètre exclu :
- Outils et commandes :
- Constats classés par gravité :
- Corrections vérifiées :
- Risques résiduels et recommandation :
- Date et signature :
