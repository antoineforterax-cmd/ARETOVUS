# 3. Télécharger et installer

[Sommaire](README.md) · [← Économie](02-economie.md) · [Réseau local →](04-reseau-et-minage.md)

## Choisir l’archive

La [Release expérimentale de référence](https://github.com/antoineforterax-cmd/ARETOVUS/releases/tag/testnet-v0.0.1-experimental.1) propose les archives du commit `421d443370e83a9ffa0c128da9f09d6f99b0ef8f` et un fichier externe `SHA256SUMS.txt`.

| Ordinateur | Archive de cette Release |
| --- | --- |
| Windows x86-64 | `aretovus-testnet-windows-x86_64-421d443370e8.zip` |
| Linux x86-64 | `aretovus-testnet-linux-x86_64-421d443370e8.zip` |
| macOS Apple Silicon | `aretovus-testnet-macos-arm64-421d443370e8.zip` |

Ne choisissez pas une archive uniquement parce que son nom contient votre système : l’architecture doit aussi correspondre. Cette publication ne fournit pas de binaire Windows 32 bits ou macOS Intel. Elle ne constitue pas une certification de toutes les anciennes versions de Windows ou de Linux.

Les programmes sont non signés. Le SHA-256 permet de comparer le téléchargement au fichier publié; il ne remplace ni une signature de l’éditeur, ni un audit. Ne désactivez pas les protections du système si elles refusent l’exécution : conservez le message pour examiner la cause.

## Vérifier et extraire sous Windows

Téléchargez l’archive Windows et le `SHA256SUMS.txt` externe dans le même dossier. Dans PowerShell, depuis ce dossier :

```powershell
Get-FileHash -Algorithm SHA256 .\aretovus-testnet-windows-x86_64-421d443370e8.zip
Get-Content .\SHA256SUMS.txt
```

Comparez les 64 caractères du résultat avec la ligne portant exactement le nom de l’archive. Les lettres majuscules et minuscules de cette empreinte ont le même sens. N’exécutez pas un téléchargement dont l’empreinte diffère.

Extrayez ensuite dans un **nouveau dossier**, par exemple `C:\Users\PC\Desktop\ARETOVUS\version-421d443`. Si un essai automatisé utilise déjà `C:\Users\PC\Desktop\ARETOVUS`, ne remplacez pas son installation ou ses données. Une archive Release s’extrait directement; les artefacts Actions historiques pouvaient contenir une seconde archive imbriquée.

Depuis le dossier extrait, qui contient `bin` :

```powershell
.\bin\aretovus-node.exe --help
.\bin\aretovus-wallet.exe --help
```

Ces deux commandes doivent afficher l’aide. Vous n’avez pas besoin de Rust ni de Visual C++ Build Tools pour exécuter ces binaires déjà construits. L’éditeur de liens Visual C++ concerne la compilation depuis les sources.

## Sous Linux ou macOS

Vérifiez l’empreinte de l’archive avec `sha256sum` sous Linux ou `shasum -a 256` sous macOS, en la comparant au fichier externe publié. Après extraction, si les droits d’exécution ont été perdus :

```sh
chmod +x bin/aretovus-node bin/aretovus-wallet
./bin/aretovus-node --help
./bin/aretovus-wallet --help
```

Les commandes de ce guide gardent les mêmes options. Par exemple, `--data-dir .\demo\noeud-a` devient `--data-dir ./demo/noeud-a`.

## Les fichiers à reconnaître

| Emplacement | Rôle |
| --- | --- |
| `bin/` | Nœud et portefeuille. |
| `docs/` | Guides fournis avec l’archive. |
| `scripts/` | Lanceur d’endurance et aides de test; Python est nécessaire pour les exécuter. |
| `SHA256SUMS.txt` dans l’archive | Empreintes des fichiers empaquetés. Il est distinct du fichier externe qui vérifie les archives. |
| Dossier passé à `--data-dir` | Données locales créées au lancement du nœud. |
| Fichier `.aretkey` créé par le wallet | Clé du portefeuille, à sauvegarder séparément des données du nœud. |

Les exécutables du nœud et du wallet ne demandent pas Python pour leurs commandes ordinaires. Le lanceur d’endurance en Python est un outil distinct.

## Pour compiler soi-même

Le [guide de contribution](../../CONTRIBUTING.md) donne la procédure. Avec les sources au commit choisi, Rust et les outils de compilation de votre système :

```sh
cargo build --workspace --release --locked
```

Les binaires sont alors dans `target/release/`. Conservez le commit et `Cargo.lock` pour identifier votre construction. La réussite d’un build local ne vaut pas audit, et les résultats CI d’un autre commit ne prouvent pas votre modification.
