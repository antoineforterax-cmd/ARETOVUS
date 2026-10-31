# Essai Linux dans Ubuntu/WSL

Utiliser une installation Ubuntu/WSL autorisée sur le PC. Cette procédure ne modifie ni Device Guard ni les protections Windows. Les binaires Linux sont construits depuis les sources; les exécutables Windows bloqués ne sont pas utilisés. Si une stratégie interdit aussi WSL, demander une autorisation à son administrateur.

Dans le terminal Ubuntu, vérifier les outils :

```sh
command -v git curl cc python3 tmux
```

Si des outils manquent, l'administrateur Ubuntu peut installer `build-essential`, `git`, `curl`, `python3`, `tmux` et `ca-certificates` avec le gestionnaire de paquets de la distribution. Ne jamais communiquer le mot de passe administrateur à un agent.

Installer Rust depuis les instructions officielles de https://rustup.rs, charger `. "$HOME/.cargo/env"`, puis vérifier `cargo --version`. Cloner le dépôt public sans identifiant ni jeton :

```sh
cd ~
git clone https://github.com/antoineforterax-cmd/ARETOVUS.git
cd ARETOVUS
git switch feature/emission-schedule-v0
git rev-parse HEAD
cargo build --release --locked
ls target/release/aretovus-node target/release/aretovus-wallet
python3 scripts/testnet_endurance.py --help
```

Conserver le commit affiché et vérifier que sa CI est verte avant la campagne. Si `ARETOVUS` existe déjà, inspecter son état Git avant de le mettre à jour; ne pas écraser des modifications. L'aide du lanceur révisé doit proposer `--wallet`; la Release expérimentale.1 contient un lanceur plus ancien.

Essai court, dans un dossier inexistant :

```sh
python3 scripts/testnet_endurance.py --binary "$HOME/AURORA/target/release/aretovus-node" --wallet "$HOME/AURORA/target/release/aretovus-wallet" --output "$HOME/essai-court" --max-rounds 6 --restart-every 3 --interval-seconds 0
cat "$HOME/essai-court/summary.json"
```

Exiger `short_run_passed`, `payments_confirmed: 5`, `final_height: 6` et `backup_restored: true`. Un échec demande une analyse avant l'essai long. Les clés créées dans le dossier sont jetables et en clair, uniquement pour le test.

Garder le PC branché et éveillé. Vérifier les redémarrages Windows planifiés; `tmux` ne survit ni à un arrêt de WSL ni à un redémarrage de Windows. Ne pas modifier la politique de sécurité du PC pour cet essai.

```sh
tmux new -s aurora-essai
python3 scripts/testnet_endurance.py --binary "$HOME/AURORA/target/release/aretovus-node" --wallet "$HOME/AURORA/target/release/aretovus-wallet" --output "$HOME/essai-72h" --hours 72
```

Détacher avec Ctrl+B puis D; revenir avec `tmux attach -t aurora-essai`. Dans un autre terminal Ubuntu :

```sh
tail -n 3 "$HOME/essai-72h/measurements.jsonl"
# Arrêt anticipé volontaire, seulement si nécessaire :
touch "$HOME/essai-72h/STOP"
```

Un arrêt anticipé n'est pas un succès de 72 heures. À la fin, conserver `summary.json`, `measurements.jsonl`, `metadata.json`, les journaux, le commit et le résultat de l'essai court. Vérifier `duration_passed`, `duration_completed: true`, `requested_hours: 72` dans les métadonnées et la restauration réussie.

Sur le poste du porteur, le chemin Windows est explicitement `PC`, pas nécessairement le nom d'utilisateur Ubuntu :

```sh
test -d /mnt/c/Users/PC/Desktop/ARETOVUS
cp -r "$HOME/essai-72h" /mnt/c/Users/PC/Desktop/AURORA/essai-72h-linux
```

Choisir une nouvelle destination si elle existe déjà. Pour partager le rapport, envoyer les JSON de mesures/résumé, le commit et le résultat court; ne pas inclure les fichiers de clés `.aurkey`. Cet essai reste un réseau de processus sur une seule machine.
