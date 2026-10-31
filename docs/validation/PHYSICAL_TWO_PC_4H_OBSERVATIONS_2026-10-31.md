# AURORA — campagne observée de 4 heures sur deux PC Windows (31 octobre 2026)

**Statut : observations de l'opérateur ; vérification des fichiers JSONL bruts et reproduction indépendante NON effectuées.** Ce document constitue un journal d'essai, **pas** une certification de robustesse ou une validation mainnet.

- Logiciel testé : testnet expérimental AURORA, commit déclaré dans la commande du collecteur sur PC B : `bbb757aaad500f8e2d51c20556edc80eacfe066e`. Le commit complet du collecteur PC A n'a pas été présenté indépendamment dans le dialogue ; l'identité binaire des deux hôtes reste à confirmer à partir des fichiers JSONL originaux.
- Topologie annoncée : deux ordinateurs Windows physiques, serveurs `server-a` et `server-b`, mineur séparé, prélèvements avec `scripts/testnet_operational_probe.py` environ toutes les 300 secondes.
- Provenance : sorties PowerShell et extraits `Get-Content ... -Tail 1`, `Select-String` et `testnet verify` **transmis par l'opérateur** le 31 octobre 2026. Les deux journaux complets ne sont **pas** intégrés dans cette PR. Aucune adresse IP, nom Windows, donnée de wallet ou clé privée publiée.

## Résultat du palier de quatre heures

| Mesure rapportée | PC A | PC B |
| --- | --- | --- |
| Durée `elapsed_seconds` | 14 401,902 s | 14 401,546 s |
| Nombre d'échantillons | 49 | 47 |
| `distinct_heights` | 45 | 42 |
| `errors` | **0** | **1** |
| `duration_reached` | true | true |
| `binary_unchanged` | true | true |
| `local_probe_passed` / `counts_as_4h` | **true / true** | **false / false** |
| Hauteur au dernier échantillon | 2 002 | 2 002 |
| Dernier `tip` | `00000e12a2c639e7663acbe79450ffae51f683495c8ee18e7586b08173aa04a9` | identique |

Après arrêt, `aurora-node testnet verify` a été exécuté sur chacun des deux hôtes : **`known_blocks=2003`** et **`work=ChainWork([2019909248, 0, 0, 0, 0])`** avec le même `tip` et la même hauteur. Il s'agit d'une **convergence finale observée**. Cela ne suffit pas à prouver un fonctionnement sans interruption ni une tolérance aux pannes prolongée.

### Incident sur PC B — ne pas masquer ni requalifier en réussite

- Échantillon `elapsed_seconds=2101.766` (UTC d'enregistrement `2026-10-31T17:34:03+00:00`) : `status=error`, `error=TimeoutExpired`, espace disque libre annoncé : 52 009 164 800 octets.
- Le collecteur impose par défaut `--command-timeout 300` à `subprocess.run([aurora-node, testnet, verify, ...])` ; l'opération ne s'est pas terminée dans le délai.
- Un contrôle ultérieur, enregistré `2026-10-31T17:44:32+00:00`, a **réussi mais pris `verify_seconds=329.188`**. Cette mesure suggère des latences transitoires sérieuses, pas un simple avertissement de configuration.
- L'opérateur a ensuite mesuré sur PC B, **au repos** et après la campagne, trois vérifications en `1.79`, `1.44` et `1.34` secondes. Cela exclut une lenteur persistante durant ces trois mesures, **pas** une défaillance intermittente.
- Le comparateur refuse à raison la campagne globale, puisque PC B a `errors=1` et `counts_as_4h=false`. Les échecs `FileNotFoundError` du comparateur lors des premiers essais provenaient de fichiers de preuves conservés **sur des ordinateurs différents**, et ne sont **pas** des erreurs de consensus.

### Contexte machine et limites causales

- PC B décrit comme un HP Laptop 17 avec AMD Ryzen 3 3200U, 8 Go de RAM installés mais **5,92 Go utilisables** ; GPU Vega 3 intégré.
- Stockage des données AURORA sur le lecteur C:, **SSD SATA LITEON CV8-8E128-HP** 128 Go déclaré `Healthy/OK`. Un second disque dur HDD Seagate 1 To est présent ; aucune preuve qu'il héberge les données testées.
- Mesures **postérieures** à la campagne : **90 %** de RAM utilisée avec navigateur/ChatGPT/autres applications ouverts, puis **73 %** après fermeture. Elles rendent plausible une pression mémoire / pagination, **mais aucune télémétrie de RAM n'a été prélevée au moment du timeout**.
- L'opérateur signale une chute passée de PC B, une charnière et un châssis endommagés, ainsi que des arrêts/redémarrages anormaux ; ces symptômes doivent être examinés séparément. Les contrôles `Healthy` et une recherche de journaux Windows sans événement correspondant ne permettent pas d'exclure une panne matérielle. Éviter de pousser ce portable tant qu'il n'a pas été sauvegardé et contrôlé.
- **Cause racine : INCONNUE**. Hypothèses à tester : contention CPU/RAM/disque, problèmes matériels intermittents, antivirus/OS, charge de synchronisation ou défaut de performance du chargeur de stockage AURORA. Ne pas attribuer l'incident à Windows ni à AURORA sans reproduction instrumentée.

## Contrôles de code et action technique

- Revue ciblée de `crates/aurora-node/src/main.rs` : `testnet verify` appelle `load_latest_snapshot`, contrôle la genèse et les paramètres, puis affiche hauteur/tip/blocs/travail. Ce parcours peut solliciter le stockage sous charge ; la revue ne prouve pas l'absence d'un défaut interne.
- Revue de `scripts/testnet_operational_probe.py` : le timeout est propagé comme `TimeoutExpired` et compté comme erreur, ce qui est **correct** ; la durée précise de l'échec et la pression mémoire système ne sont pas conservées dans la version testée.
- Correctif proposé dans la même PR : instrumentation **additive** du collecteur (temps mesuré y compris en cas d'échec, durée du délai configuré, mémoire disponible sur Windows), sans changement des règles de validation ou des formats de consensus.
- Conserver les journaux bruts localement ; pour un futur essai, sauvegarder l'état des deux machines, documenter la version binaire, et mesurer les ressources pendant le prélèvement. Ne pas relancer automatiquement quatre heures de charge sur un portable potentiellement endommagé.

## Décision

- **PC A :** critère local 4 h satisfait selon l'extrait fourni.
- **PC B :** critère local 4 h **non satisfait** (`errors=1`).
- **Deux hôtes :** convergence finale des chaînes **observée**, mais **palier conjoint 4 h NON VALIDÉ**.
- **72 h, 100k+ blocs, audit indépendant, sécurité mainnet :** aucune revendication.

_Statut documentaire : compte rendu de faits communiqués, à recouper ultérieurement contre les deux JSONL originaux et les empreintes des exécutables ; aucune donnée personnelle requise dans les PR publiques._
