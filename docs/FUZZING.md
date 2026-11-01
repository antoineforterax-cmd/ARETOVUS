# Fuzzing ARETOVUS

The main CI runs a deterministic malformed-input smoke test against transaction, block, header, block-inventory and peer-list decoders. It is a quick regression check, not a substitute for coverage-guided fuzzing.

The separate `fuzz/` workspace contains libFuzzer targets for canonical transaction/block/header decoding, Merkle proof verification, and P2P block-inventory/peer-list decoding. GitHub Actions runs each target for 5 minutes on Linux for pull requests and manual dispatches.

## Local run

The libFuzzer toolchain requires nightly Rust, a C++ compiler, and cargo-fuzz. The supported runner in CI is Linux:

```sh
rustup toolchain install nightly --profile minimal
cargo +stable install cargo-fuzz --version 0.13.2 --locked
cargo +nightly fuzz run canonical_decode -- -max_total_time=300
cargo +nightly fuzz run merkle_proofs -- -max_total_time=300
cargo +nightly fuzz run network_payloads -- -max_total_time=300
```

Corpus and crash artifacts are written under `fuzz/corpus/` and `fuzz/artifacts/`. Preserve minimized crash inputs as regression cases and fix the underlying decoder before continuing. CI time limits are deliberately short; release candidates need longer campaigns and an independent review of any findings.

## Preuves GitHub Actions et rejouabilité (B4)

Le workflow `fuzz` conserve désormais un artefact `fuzz-evidence-<run_id>` pendant **14 jours** (rétention configurable). Il contient, lorsqu'ils existent :

- `target/fuzz-evidence/provenance.txt` : événement, SHA indiqué par GitHub, SHA effectivement checkouté, référence de PR, run ID, budget par cible, version exacte du compilateur Rust nightly ;
- `canonical_decode.log`, `merkle_proofs.log`, `network_payloads.log` : sortie standard et erreurs, avec les statistiques finales de libFuzzer ;
- `fuzz/corpus/` : entrées de corpus enrichies par chaque cible et `fuzz/artifacts/` : éventuels crashs, dépassements de temps ou autres cas reproductibles.

L'étape d'upload utilise `if: always()`, y compris si un fuzzer échoue. Les commandes redirigent vers `tee` avec `set -o pipefail` : **la collecte des journaux ne masque pas un code de sortie en erreur**. Les étapes des fuzzers restent **séquentielles** ; un crash dans une cible peut empêcher les suivantes de s'exécuter : leurs journaux seront alors absents, ce qui ne vaut pas un test réussi.

La valeur `FUZZ_SECONDS` est le budget maximum libFuzzer **par cible** et non une durée d'exécution constatée ; conserver les timestamps du run. Lors d'une PR, `GITHUB_SHA` peut désigner le commit de fusion de test GitHub, différent du commit de tête de la PR : lire `checkout_sha` et les métadonnées du run avant d'attribuer les résultats à un SHA précis.

Pour reproduire un crash : récupérer l'artefact du run, vérifier le checkout et les versions Rust/cargo-fuzz, puis rejouer le **fichier concret** dans la cible correspondante, sur un clone jetable et isolé, par exemple :

```sh
cargo +nightly fuzz run canonical_decode fuzz/artifacts/canonical_decode/crash-EXEMPLE
```

`crash-EXEMPLE` n'est qu'un nom illustratif ; utiliser le chemin réel de l'artefact. Ajouter ensuite un test de régression déterministe et ne corriger ni contourner le défaut en supprimant simplement le cas. Les corpus sont générés uniquement sur des données synthétiques de CI : ne jamais y joindre des clés, sauvegardes ou identifiants réels.

**Limites :** un résultat `success` signifie seulement qu'aucune erreur n'a été remontée durant cette campagne bornée. Le replay n'est pas garanti bit pour bit sur des machines et versions différentes ; l'archivage améliore l'analyse mais n'est ni un audit indépendant ni une preuve d'absence de bug ou de sécurité mainnet.
