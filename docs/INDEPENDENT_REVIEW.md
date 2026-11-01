# Dossier de revue indépendante — ARETOVUS

**Statut :** dossier préparatoire ; aucune revue extérieure qualifiée, approfondie et documentée n’est confirmée. Le responsable a déjà relu et corrigé du code, notamment en Python et rust, et réalisé des essais : cela relève des vérifications internes, distinctes de la revue demandée ici. Le porteur a choisi le 26 octobre 2026 de poursuivre le développement expérimental sans reviewer externe immédiat. Cette décision ne vaut pas revue et ne permet pas de revendiquer une sécurité pour des fonds réels.  
**Branche concernée :** `feature/emission-schedule-v0`.  
**Base proposée pour la revue du consensus :** `7d0f1ee598235b7194f92081424c008c4c628ceb`; les changements documentaires et de lanceurs suivants doivent être examinés séparément et identifiés dans le rapport.
**Usage autorisé du code :** testnet local/expérimental uniquement; pas de fonds réels.

Une [contre-revue par un second agent IA](AI_COUNTER_REVIEW.md) a signalé quatre risques et ouvert les tâches de correction. Elle reste une aide interne; elle ne remplit pas le critère de revue humaine externe indépendante.

## Portée demandée

La revue devrait couvrir le code et le protocole au commit exact enregistré par l’auditeur :

1. Encodage canonique, identifiants de transaction et domaine de hachage.
2. Vérification ML-DSA-65, construction/signature de transaction, nonces, soldes et cas de rejeu.
3. Validation des blocs, engagements Merkle, limites de taille et preuve de travail SHA-256.
4. Retarget, règles d’horodatage/MTP, travail cumulé, égalités de travail et réorganisations.
5. Émission, plafond brut, trésorerie 5 %, frais 50/50, burns et état lors d’une réorganisation.
6. Décodage P2P/RPC, quotas, délais, synchronisation, découverte et gestion de données hostiles.
7. Stockage paginé v6, lecture des snapshots v4 et index v5, checksums, rejeu de toutes les branches, reprise, verrou exclusif de dossier entre processus et erreurs d’E/S.
8. Wallet testnet, fichiers de clés ARETOVK4 chiffrés par défaut (Argon2id et XChaCha20-Poly1305), sauvegarde/restauration, format clair explicite réservé aux tests, URI et vérification SPV contre éclipses et données de pairs mensongères.

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
- La chaîne conserve le sommet canonique et des points de contrôle d’état tous les 256 blocs; les autres états sont rejoués depuis un ancêtre connu. Le coût des états complets aux points de contrôle, du rejeu et de la copie de chaîne par page de synchronisation reste à mesurer avec de nombreux comptes et branches.
- Les tests de snapshot ne simulent pas une panne électrique réelle, le disque plein, ni une restauration interversion.
- Le SHA-256 utilisé pour les identifiants et le PoW n’est pas une construction résistante aux ordinateurs quantiques.

## Voie pratique sans engager d’audit payant

Le dépôt est public depuis le 27 octobre 2026. Un étudiant, un développeur indépendant ou un passionné ayant les compétences nécessaires peut lire la branche, examiner la PR et remettre des constats reproductibles sans invitation ni droit d’écriture. Aucun diplôme, poste ou cabinet d'audit n'est exigé pour contribuer ; ce sont la méthode, la rigueur et la capacité à vérifier les corrections qui comptent. Cette voie dépend de sa disponibilité et ne garantit ni une revue gratuite ni sa qualité.

Le porteur a choisi users.rust-lang.org et les Discussions GitHub pour un appel à relecture. Le [texte préparé](HUMAN_REVIEW_CALL.md) n’a pas encore été publié : l’accès à une session authentifiée n’est pas disponible dans cette intervention. Quand un reviewer se propose, lui communiquer le périmètre et le commit, conserver ses constats écrits, corriger les défauts puis demander la vérification des correctifs. Une auto-revue, un second outil automatique ou une autre session d’IA peuvent aider à trouver des défauts, mais ne remplissent pas cette étape humaine indépendante.

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
