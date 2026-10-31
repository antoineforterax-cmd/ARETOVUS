# AURORA — coût des snapshots pendant une synchronisation P2P

_Date : 31 octobre 2026 — diagnostic du testnet Regtest, aucun changement de consensus._

## État réellement mesuré

| Hauteur TCP | SHA testé | Sessions TCP | Sync + sauvegarde | Dont sauvegarde | Recharge | Source |
| ---: | --- | ---: | ---: | ---: | ---: | --- |
| 32 768 | `6f4b9b1956b49fb1049595d84191646a1133eb03` | 74 | 116,094 s | 78,703 s | 9,17 s | [Run 41212675879](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41212675879) |
| 65 536 | `4576f7dd692a0dbef4ee4883b1513ee8f38c07fd` | 147 | 355,390 s | 271,128 s | 18,76 s | [Run 41213036562](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41213036562) |

Deux runs GitHub distincts, pas un protocole A/B/A/B sur un runner fixe ; la variation du matériel et du disque peut influer. Ce sont des tests **TCP loopback Regtest**, avec branche secondaire et restauration, pas un essai sur deux PC physiques. La RSS `/usr/bin/time -v cargo test` n'isole pas le processus nœud.

Le test TCP à 100 001 blocs a été lancé sur `main` SHA `8643bab287b0d1a89c38e35affd64f58c5861a52`, via la [PR #116](https://github.com/antoineforterax-cmd/AURORA/pull/116). Ne pas considérer ce palier réussi sans la conclusion et les logs du **workflow push** qui sélectionne 100 001 (une PR ordinaire exécute seulement 2 048).

## Modèle de coût déduit du code

Dans `sync_from_peer_with_admission`, chaque page de `PAGE_BLOCKS = 64` est validée, puis `save_snapshot_with_page_cache` persiste le candidat avant de publier la chaîne. `save_snapshot_inner` appelle `symlink_metadata` sur **tous** les fichiers blocs connus à **chaque** snapshot, même lorsque le cache accélère la vérification des pages d'index. Ce comportement détecte notamment la substitution d'un ancien bloc par un lien symbolique.

Avec une chaîne initiale d'un bloc de genèse, des pages de 64 nouveaux blocs et sans branches latérales, le nombre théorique **minimal de contrôles de chemins** est :

```text
C(N) = somme_{j=1..ceil(N/64)} (1 + min(64*j, N))
```

| Blocs synchronisés N | Snapshots | Contrôles de chemins prévus C(N) |
| ---: | ---: | ---: |
| 8 192 | 128 | 528 512 |
| 32 768 | 512 | 8 405 504 |
| 65 536 | 1 024 | 33 588 224 |
| 100 001 | 1 563 | 78 226 556 |

Les branches secondaires et les autres écritures peuvent augmenter ce volume. Le nombre d'appels croît en **O(N² / 64)** pour une page fixe de 64 blocs, mais il serait erroné d'en déduire que toute la durée CPU ou disque suit exactement la même loi.

## Instrumentation ajoutée dans la PR #117

Les profils de compilation de test/débogage avec `AURORA_STORAGE_PROFILE=1` exposent : contrôles de métadonnées existantes, nouveaux objets blocs, pages réutilisées, pages non cachées, coût de la phase blocs, de l'index, de la publication et des deux étapes de purge. Aucun compteur n'autorise à sauter une vérification. La CI dédiée mesure 8 192 et 32 768 blocs TCP sur un même runner avec contrôle du résultat et artefacts de logs ; les chiffres ne sont **pas** des mesures avant la fin de la CI au SHA exact.

## Options et décisions de sûreté

1. **Conserver le comportement actuel** pendant l'établissement des références et des tests adversariaux. Les PR #110, #111 et #114 sont déjà intégrées à main ; leur protection contre la substitution locale ne doit pas être perdue.
2. **Optimiser les coûts secondaires** (encodage de chemins, constructions répétées, ordre des contrôles, purge) seulement après profilage A/B/A/B, sans déplacer les points de publication ni réduire les vérifications.
3. **Modifier la fréquence de snapshot** (par exemple persister par lot de pages) est un choix distinct : il peut réduire le nombre de scans, mais change la granularité de reprise et le moment de publication. Exige une décision écrite du responsable et des tests d'interruption/session/reconnexion/partitions/disque plein avant tout code.
4. **Éviter de revérifier les anciens objets** suppose des garanties nouvelles de propriété/immutabilité du répertoire et un modèle de menace différent : interdit de présenter un cache RAM comme preuve de validité persistante. Demande une proposition d'architecture et un accord explicite.

**Gates restants :** campagne longue >100k, 72 h sur machines réelles, test multi-hôte, audit humain indépendant, corruption et crash sur snapshots combinés, comparaison des gains avec incertitude. Aucun usage de fonds réels.
