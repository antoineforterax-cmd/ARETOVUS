# AURORA — plan de persistance incrémentale sans perte de sûreté

**Statut : proposition de conception, non implémentée.** Référence : `main` après la fusion #107, SHA `e147629f7deb430ac193052de1a4864f8fa6ac40`. Aucun paramètre de consensus, format de disque ou règle de récupération n'est approuvé par ce document. Ne pas utiliser de fonds réels.

## 1. État mesuré et raison du chantier

Le chemin `sync_from_peer_with_admission` (`crates/aurora-node/src/p2p.rs`) clone la chaîne, valide chaque page TCP, appelle `save_snapshot(data_dir, &candidate)` **avant** de publier le candidat, et n'incrémente la chaîne visible qu'après succès de la persistance. Le rejet d'une page hostile ne doit pas détruire les pages précédemment validées.

Le `save_snapshot` v6 (`crates/aurora-storage/src/lib.rs`) :
1. parcourt `blocks_with_ids_by_arrival()` dans l'ordre d'arrivée et fait un `symlink_metadata` **pour chaque chemin** `blocks-v5/<hash>.aurblk` ; écrit les objets absents par fichier temporaire, `sync_all`, `rename`, synchronisation du répertoire ;
2. reconstruit chaque page de 4 096 hashes dans `index-pages-v6/`, son hash de contenu, puis le manifeste avec ses hashes de pages ;
3. synchronise les répertoires d'objets avant de publier le manifeste checksummé, puis ne purge les générations que sous des contrôles conservateurs ;
4. au redémarrage, revalide manifestes, pages, blocs et consensus, et retient un point de reprise encore valide.

Les blocs de branches concurrentes restent dans l'ordre d'arrivée local ; le tip canonique et l'état sont recalculés au rejeu. Le cache de la PR #107 est **éphémère au seul passage** de vérification des pages de rétention.

**Preuves historiques à ne pas extrapoler :**
- [PR #104, TCP loopback 100 001 blocs](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41202647665), SHA `22cec763...` : `save_seconds=1016.056`, 1 563 pages, 224 sessions, 100 003 blocs connus (branche secondaire incluse), restauration 29,21 s. Test local, pas deux PC.
- [PR #107, A–B–B–A](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41206941481) à 32 768 blocs : 125,230 s → 95,500 s de construction + sauvegarde (−23,7 %), avec reprise réussie.
- [PR #108, A–B–B–A](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41208168717) à 32 768 blocs : 91,575 s → 90,510 s (−1,2 %), non significatif à deux répétitions. La PR #108 reste en brouillon et **ne doit pas être fusionnée comme une optimisation démontrée**.

## 2. Invariants non négociables

| Invariant | Critère de succès |
| --- | --- |
| Publication atomique | Une page n'est visible dans `chain` qu'après validation complète et persistance du manifeste qui l'engage |
| Dernier point de reprise | Erreur d'écriture, crash ou ENOSPC préserve au moins le dernier snapshot entièrement engagé et les fichiers nécessaires à sa restauration |
| Données immuables | Aucun objet symlink / dossier / fichier manquant ne doit être silencieusement considéré présent ; ne pas suivre de lien symbolique par commodité |
| Corruption | Pages/manifests incorrects refusés ; au redémarrage les objets blocs et la chaîne sont intégralement revalidés |
| P2P hostile | Page répétée, tronquée, contradictoire ou invalide ne mutile pas l'état précédemment accepté ; limite des ressources, curseur u64, continuité de session inchangés |
| Branches et réorganisations | Toutes les branches sont persistées et restaurées dans l'ordre d'arrivée ; le meilleur sommet reste décidé par les règles existantes |
| Durabilité | Ordre temp → write → sync fichier → rename → sync répertoire → publication du manifeste ; ne jamais remplacer les `fsync` par du cache mémoire |
| Compatibilité | Même format index/pages/manifeste v6 ; lecture v4/v5 conservée ; pas de changement consensus, de genèse, d'émission, de frais ni d'encodage réseau |
| Exhaustivité | La vérification d'un objet ne doit pas disparaître parce que le hash figurait dans un manifeste antérieur |
| Échec fermé | Un cache périmé, un ordre d'arrivée altéré, un dossier inattendu ou une impossibilité de valider une hypothèse renvoie au `save_snapshot` classique |

**Point crucial :** on ne peut pas à la fois **ne plus examiner un ancien chemin de bloc** et prétendre détecter avec la même certitude, à chaque sauvegarde, sa substitution externe par un symlink ou un dossier. Un changement non observé sur un chemin non inspecté produit le même état du cache qu'un disque intact. La sûreté complète exige donc soit (a) de conserver son examen, soit (b) une garantie d'immuabilité du filesystem ou un mécanisme d'observation fiable et démontré. Les notifications filesystem seules, sujettes aux pertes et débordements, **ne constituent pas cette preuve**. Aucune baisse du nombre de `symlink_metadata` n'est autorisée dans la première phase.

## 3. Plan d'implémentation progressif

### Phase A — optimisation compatible et conservatrice, sans nouvel index disque

**Objectif :** diminuer les allocations, la reconstruction des pages stables et certaines copies, sans jamais sauter l'examen des fichiers blocs. Une seule PR de code à la fois, issue de `main`, avec reproduction et mesure ABBA avant/après.

- Préparer un cache **en mémoire, non persistant**, d'identifiants de pages v6 déjà construits : `{start, count, page_hash, expected_block_ids}` ou une représentation équivalente bornée. Le cache est associé à une session de synchronisation **et à un préfixe exact de l'ordre d'arrivée**, jamais seulement à un nombre de blocs.
- Seules les pages entièrement stables de 4 096 éléments sont susceptibles d'être réutilisées. Une page terminale partielle est recalculée quand de nouveaux blocs arrivent. Les nouvelles branches ajoutent des éléments à l'inventaire ; elles ne peuvent pas insérer un hash au milieu. En cas de divergence du préfixe, d'interruption, de redémarrage ou d'échec d'écriture : **invalider le cache et repasser par le chemin complet**.
- Reconstituer le manifeste à partir des hashes des pages précédentes déjà **vérifiés** et des pages nouvelles. Avant toute publication, contrôler la présence, le type, la taille et le hash du contenu de chaque page réutilisée *pendant cette même opération* ; les garanties doivent au minimum égaler le `write_immutable` courant, qui relit les pages existantes.
- Parcourir **tous les blocs**, conserver le `symlink_metadata` de chacun, et conserver l'écriture sûre des objets manquants ; l'optimisation ne doit pas convertir une corruption en cache hit.
- Ne jamais publier un candidat dont l'un des objets ou pages est invalide. Les `sync_directory` précédant la publication restent obligatoires, y compris sur les répertoires dont les éléments étaient déjà présents.
- Si l'économie de reconstructions est noyée par le coût des `symlink_metadata`, ne pas prétendre que les sauvegardes sont désormais sous-linéaires. Produire les chiffres et chercher un autre gain compatible.

**Portée prévue :** un module interne restreint du crate `aurora-storage` et un test de scénario dans `aurora-node`. Sans modification du format, l'API publique classique de sauvegarde reste le comportement de référence.

### Phase B — recherche sur une véritable voie incrémentale de fichiers blocs (PAS AUTORISÉE POUR PRODUCTION)

**Question d'architecture à trancher avant tout changement :** le modèle de menace protège-t-il contre les altérations par d'autres processus locaux entre deux sauvegardes, ou suppose-t-il que le stockage est gardé immuable et exclusif par le système d'exploitation ?

Pour une voie réellement O(nouveaux blocs) lors des sauvegardes :
- Isoler chaque génération dans une propriété de répertoire ou une capacité de dossier **réellement protégée par l'OS**, pas simplement une convention applicative. La preuve doit couvrir Linux, Windows et macOS ; le verrou de nœud ordinaire n'empêche pas un programme externe de remplacer des fichiers.
- Définir comment déceler les modifications externes et comment annuler l'optimisation avant publication. Un watch `inotify`/FSEvents/ReadDirectoryChangesW avec débordement, perte après crash, déplacement du répertoire ou volume non supporté doit forcer une vérification complète ; cela ne couvre pas à lui seul toutes les races.
- Maintenir une opération de vérification complète indépendante au redémarrage et après incident. Si la détection **à chaque sauvegarde** est affaiblie, cela change la promesse de sûreté ; demander une **décision de sécurité explicite** et les bornes de perte acceptables, sans fusion automatique.
- Ne pas introduire de journal disque additionnel ou version v7 sans spécification de crash, compatibilité et migration distinctes.

**Aucune implémentation de Phase B** tant que ces hypothèses de sécurité ne sont pas établies. Une simplification qui saute des vérifications parce qu'un bloc « devrait être là » est refusée.

### Phase C — observation et mesure sans extrapolation

- Les tests ciblés doivent instrumenter séparément : vérifications d'objets déjà présents, objets nouveaux, pages v6 stables reconstruite/réutilisées, `fsync`, rétention, clone mémoire, publication P2P, replay et RSS du **processus ciblé** lorsque possible.
- Comparer même SHA de base et SHA candidat sur le **même runner** à volumes 8 192 puis 32 768, ordre ABBA, **au moins deux essais chacun** ; ajouter un volume 100 001 sur commit final en test TCP loopback avec fourchette de ressources documentée.
- Contrôler : hauteur, hash du tip, total de blocs connus y compris fork, nombre de sessions TCP, coût par phase, restauration après restart, aucun test filtré à zéro.
- Une variation de 1 % avec deux répétitions n'est pas un argument suffisant de fusion. Un gain visible doit subsister après le coût de tous les contrôles de sûreté.

## 4. Matrice de tests de régression à écrire avant la voie rapide

| Scénario | Attendu |
| --- | --- |
| Bloc ancien remplacé par symlink avant sauvegarde suivante | Rejet immédiat, ancien snapshot récupérable ; aucune publication |
| Bloc ancien supprimé entre deux pages | Réécriture atomique et complète, sans détournement de chemin |
| Bloc ancien remplacé par dossier | Rejet avant publication |
| Page d'index déjà présente modifiée ou tronquée | Rejet ou reconstitution *vérifiée* avant référencement ; ne jamais utiliser le hash en cache sans validation |
| Page terminale partiellement remplie puis complétée | Nouveau hash de page et manifeste cohérent ; pages de préfixe stables inchangées |
| Fork reçu tardivement + réorganisation profonde | Compte et ordre d'arrivée stricts, sommet correct, restauration de branche perdante |
| Paire P2P contradictoire à la deuxième page | Rejet atomique sans perdre la première page déjà persistée |
| Crash entre `rename` et `sync_directory` | Fichier orphelin toléré ou récupération de la dernière génération complète, jamais de manifeste référençant un objet non durable |
| ENOSPC dans bloc/page/manifeste/retention | Retour au dernier snapshot restaurable, reprise après libération de l'espace |
| Deux nœuds sur le même dossier / permissions changées | Respect du verrouillage et des erreurs ; jamais de cache présumé valide |
| Mutation externe pendant une vérification | Ne pas détruire l'ancre valide ; documenter le résultat exact sous le modèle de menace choisi |
| Restart, copie hors runner, Windows/Linux/macOS | Réexamen complet et compatibilité v4/v5/v6 préservée |
| Cache consommant une mémoire excessive | Limites explicites, invalidation sûre et retour à `save_snapshot` |

## 5. Gates avant code puis avant fusion

1. Référencer **le SHA exact de `main`** et le résultat de sa CI, ainsi que les PR encore ouvertes et leurs dépendances. La PR #108 n'est pas fusionnée ; son gain mesuré est faible.
2. Ajouter les tests de régression négatifs au **comportement classique** ; si un test reproduit un défaut de sûreté, corriger d'abord ce défaut dans une PR distincte, en respectant `SECURITY.md` pour les vulnérabilités.
3. Implémenter **Phase A uniquement** sur une branche indépendante. CI : `cargo fmt`, Clippy, build locked, tests, smoke, ENOSPC, backup inter-runner, partitions P2P et fuzzing pertinent ; A/B au SHA exact.
4. Réviser manuellement le modèle de menace, les bornes de mémoire, les races, la compatibilité et les erreurs de rejeu. Revue par l'auteur-assistant **≠** contre-expertise indépendante.
5. Attendre l'**accord explicite** avant de fusionner la branche de code. La fusion elle-même nécessite une CI sur le **nouveau commit de `main`**.

## Décision proposée

**Autoriser uniquement la phase A** comme prochaine expérimentation, sans modification de l'engagement de sécurité des fichiers blocs. Reporter tout « skip des blocs historiques » (Phase B) jusqu'à démonstration de l'immuabilité externe ou choix explicite de modèle de menace. Le testnet reste expérimental, non audité, et ne doit pas être utilisé avec des fonds réels.
