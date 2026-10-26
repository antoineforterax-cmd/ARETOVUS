# Dossier de revue indépendante — AURORA

**Statut :** dossier préparatoire; aucune revue indépendante n’a été réalisée. Le porteur a choisi le 26 octobre 2026 de poursuivre le développement expérimental sans reviewer externe immédiat. Cette décision ne vaut pas revue et ne permet pas de revendiquer une sécurité pour des fonds réels.  
**Branche concernée :** `feature/emission-schedule-v0`.  
**Usage autorisé du code :** testnet local/expérimental uniquement; pas de fonds réels.

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

- Les paramètres de difficulté du testnet sont volontairement faibles et ne protègent pas des fonds réels.
- Le nœud utilise une topologie amorcée par des adresses connues avec rattrapage et relais périodique des transactions depuis les pairs configurés; annonce spontanée à des pairs non configurés et découverte multi-sauts ne sont pas fournies.
- Le portefeuille chiffre ses nouvelles clés par défaut, mais l’ergonomie du mot de passe, la dérivation, la sérialisation du fichier et la sauvegarde/restauration n’ont pas été revues indépendamment. Le format en clair est réservé aux scripts de test; ne jamais utiliser ces clés hors testnet.
- La preuve SPV dépend des pairs consultés et ne défend pas contre une éclipse.
- La trésorerie reçoit des unités à une adresse fixe, mais aucune gouvernance ni règle de dépense n’est définie.
- Les tests de snapshot ne simulent pas une panne électrique réelle, le disque plein, ni une restauration interversion.
- Le SHA-256 utilisé pour les identifiants et le PoW n’est pas une construction résistante aux ordinateurs quantiques.

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
