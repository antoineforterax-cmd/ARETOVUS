# AURORA — campagne de montée en charge P2P (Regtest local)

Ce document concerne les **essais de synchronisation TCP** du nœud expérimental.
Il ne constitue ni un audit de sécurité ni une mesure du débit du Testnet
public (PoW, plusieurs machines, latence WAN et profils de signature différents).

## Reproductibilité

- Source : `crates/aurora-node/src/p2p.rs`, test Rust
  `p2p::tests::network_volume_tcp_sync_with_side_branch_and_disk_replay`.
- Lanceur : `.github/workflows/p2p-volume-probe.yml`, runner GitHub
  `ubuntu-latest`, Rust 1.99.1, limite 45 minutes, logs artefacts conservés 14 jours.
- Préparer une chaîne Regtest à difficulté fixe 0, puis une branche concurrente.
  Transférer par **vraies connexions TCP loopback**, persister les pages dans un
  dossier différent et vérifier hauteur, tip, blocs connus, branche et
  restauration après arrêt du client.
- Forcer huit requêtes maximum par session de test et exiger plusieurs
  sessions TCP : cela exerce la reprise du curseur entre les sessions.
- Mesurer séparément `build_seconds`,
  `tcp_sync_and_page_persist_seconds`, `restore_seconds`,
  `total_seconds` et le RSS maximal de la **commande Cargo**.
  Ce RSS inclut compilation/test et n'est pas un pic mémoire isolé du nœud.
- Le workflow vérifie **un test exécuté** avec `1 passed; 0 failed`.
  Un lancement dont le filtre Cargo exécute zéro test est un échec de campagne,
  même si Cargo renvoie `0`.

## Résultats vérifiés

| Campagne | SHA | Blocs connus | Sessions TCP | Synchronisation + persistance | Restauration | Preuve |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| Baseline après #64 | `0e0bb6798d45f939b9d111c77c56105a89c6c6ff` | 2 050 | 5 | 3,70 s | 0,40 s | [Actions 41059067553](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41059067553) |
| Palier 8 192, branche de validation | `3b85bdc47c840adf53a7619890de64127a091a0e` | 8 194 | 19 | 37,27 s | 1,79 s | [Actions 41060240777](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41060240777) |
| Palier 16 384, branche de validation | `e966fc1dc8d2d75fc4018ed2719bac217ba379e1` | 16 386 | 37 | 79,23 s | 4,48 s | [Actions 41061566814](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41061566814) |

Avec ~4 fois plus de blocs, la synchronisation + persistance passe de
3,70 s à 37,27 s (~10,1 fois). Cette mesure suggère un coût non linéaire :
le chemin P2P clone et persiste la chaîne lors de l'acceptation de chaque
page. Ce n'est **pas** une preuve isolée de la cause : mesurer séparément
validation, copies en mémoire et `save_snapshot` avant d'optimiser.
Le RSS de la commande Cargo à 8 192 blocs : **371 368 Ko** ; il inclut la
compilation, pas uniquement le nœud.


Les mesures d'autres exécutions du même code varient avec les runners et la
charge système : les valeurs ci-dessus ne constituent ni un SLA, ni une
garantie d'évolution linéaire.

À 16 384 blocs : génération 1,31 s, **79,23 s** synchronisation
et persistance par pages, 4,48 s restauration, **37 sessions TCP**,
85,37 s scénario complet. La commande Cargo a duré 1 min 38,13 s avec
RSS maximal annoncé **370 188 Ko** (compilation incluse).
Le palier 16 384 est environ deux fois plus grand que le palier 8 192,
et son temps TCP + persistance est ~2,13 fois supérieur sur ces runners.
Ces deux mesures ponctuelles ne prouvent pas une loi de complexité,
ni une durée probable à 100 001 blocs.


## Paliers opt-in

| Palier | Déclenchement | Critère de validation |
| --- | --- | --- |
| 2 048 blocs de hauteur (2 050 connus) | PR et déclenchement manuel par défaut | Un test exact, convergence, branche latérale, restauration, plusieurs sessions |
| 8 192 blocs de hauteur (8 194 connus) | Push de `volume/p2p-scale-8192` | Même invariant ; conserver temps, RSS et verdict GitHub au SHA |
| 16 384 blocs de hauteur (16 386 connus) | Push de `volume/p2p-scale-16384` | Exécution validée sur `e966fc1dc...`, vérifier CI du commit final de la PR |
| 100 001 blocs de hauteur (100 003 connus) | `workflow_dispatch` explicite avec `blocks=100001` | Ne déclarer réussi qu'après exécution complète ; ne pas extrapoler |

Les branches à paliers permettent une campagne sur demande sans ralentir
`main` ni tous les changements ordinaires. Le workflow de PR reste au palier
2 048, pour tester la régression sur le code exact.

## Limites et risques connus

`sync_from_peer_with_request_limit` clone l'état de la chaîne et appelle
`save_snapshot` à chaque page. Ces opérations peuvent rendre la durée
**superlinéaire** avec le nombre de blocs. Le test Regtest ne mesure pas le
comportement de deux machines physiques, la charge réseau hostile, une
interruption au milieu d'une trame ou les règles de consensus Testnet.
Ne pas augmenter les quotas P2P ou désactiver la validation pour accélérer
le benchmark : une éventuelle optimisation doit faire l'objet d'une PR
séparée, avec tests adversariaux et invariants préservés.
