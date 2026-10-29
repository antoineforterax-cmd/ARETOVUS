# Contre-revue P2P — inventaires répétés

Analyse commencée le 27 octobre et complétée le 28 octobre 2026 par un sous-agent IA. Base examinée : `421d443370e83a9ffa0c128da9f09d6f99b0ef8f`, puis correctif local dans `crates/aurora-node/src/p2p.rs`. Ce document ne constitue pas une revue humaine indépendante. Il ne revendique pas de validation CI du correctif avant qu'un commit et ses résultats y soient ajoutés.

## Défaut identifié

Le protocole v4 expose un inventaire de blocs en ordre d'arrivée : un hash de bloc occupe une seule position. Le client accepte pourtant `DuplicateBlock` lors du traitement d'une page, car un bloc reçu peut légitimement être déjà présent dans sa chaîne locale.

Ces deux cas doivent être distingués :

- Un bloc local apparaît une fois dans l'inventaire distant : cas normal.
- Le pair répète le même bloc à plusieurs curseurs de son propre inventaire : inventaire incohérent.

Avant le correctif, le second cas faisait avancer le curseur sans nouveau bloc. Un pair pouvait annoncer un total très grand, répéter un petit ensemble de blocs valides et faire renouveler les budgets de session. Le client n'avait pas de contrôle reliant cette progression à des positions distinctes. Dans le serveur, la tâche de synchronisation parcourt les pairs séquentiellement; un appel qui ne finit pas retarde les pairs suivants. Cela ne permet pas de créer des fonds ni de faire accepter un bloc invalide, mais fournit un moyen de monopoliser le rattrapage.

Le constat provient de la lecture du chemin `sync_from_peer_with_request_limit` et des tests de régression ajoutés. Une tentative de reproduction sur le binaire Windows de la Release `421d443` a échoué à établir sa connexion locale avec l'erreur Windows **10013** dans l'environnement d'exécution restreint. Elle n'est donc pas comptée comme une reproduction réussie. Le processus local de cette tentative a été arrêté; aucun hôte externe n'a été contacté.

## Correctif

Le client mémorise les hashes correspondant aux positions déjà traitées dans cet inventaire, y compris après une reconnexion. Il rejette une répétition à un nouveau curseur avant de cloner ou publier l'état de la page. Le hash de la genèse est réservé à la position zéro. La sonde de continuité à la reconnexion relit volontairement la dernière position; elle n'ajoute pas une nouvelle entrée à cet ensemble et reste autorisée.

La vérification provisoire est limitée à une page de **64 blocs au maximum**. Les entrées de la page ne rejoignent l'ensemble durable de l'appel qu'après validation de tous ses blocs et, lorsqu'elle est demandée, leur sauvegarde. Aucune capacité n'est allouée à partir du total `u64` annoncé par le pair. Le nombre de hashes conservés est au plus celui des blocs validés connus localement; une assertion en mode debug vérifie cet invariant. Le surcoût de mémoire est donc **O(blocs connus)** pendant cet appel, pas une limite constante. Il n'ajoute pas de seuil arbitraire à 100 000 blocs.

Les règles de consensus et l'encodage réseau restent inchangés. Les pairs honnêtes produisent déjà un inventaire unique. Les pages précédentes correctement validées restent appliquées si une page ultérieure est rejetée; la page rejetée ne doit modifier ni la chaîne ni le disque.

## Tests ajoutés

| Test | Ce qu'il vérifie |
| --- | --- |
| `repeated_block_in_one_sync_page_is_rejected_before_persistence` | Un pair TCP local fournit deux fois le même bloc dans une page. L'erreur attendue est explicite; hauteur, sommet et dossier de stockage restent inchangés. |
| `repeated_inventory_block_is_rejected_across_pages_and_session_rollover` | Un bloc déjà connu localement est légitime à sa première position distante, puis refusé à une deuxième position. Le scénario est exécuté avec et sans reconnexion, avec un total annoncé égal à `u64::MAX`. |
| `sync_accepts_remote_inventory_that_overlaps_local_blocks` | Cas positif : un serveur fournit un bloc déjà présent et un nouveau bloc. Le client termine normalement et ne compte que le nouveau bloc. |

Le test existant `sync_reopens_session_and_checks_last_cursor_before_continuing` vérifie aussi la reprise légitime après la sonde de continuité. Tous ces scénarios restent locaux et utilisent des données Regtest jetables.

Vérifications effectuées dans cette intervention : formatage Rust du fichier et `git diff --check`, sans erreur. La compilation et l'exécution des nouveaux tests restent à confirmer par CI; les obstacles locaux au linker Windows sont décrits dans le rapport général.

## Risques qui restent ouverts

- Un pair peut encore fournir beaucoup de blocs distincts et valides ou une suite d'inventaires croissants. Ce correctif ne définit pas une politique de rétention des branches, un budget global de rattrapage ou un ordonnancement équitable entre pairs.
- Le client clone la chaîne par page et conserve des états aux points de contrôle. Le coût reste à mesurer avec de nombreux comptes et branches; le suivi anti-répétition ajoute un ensemble temporaire de hashes.
- Les huit workers et la limite de quatre sessions par adresse IP ne protègent pas d'un adversaire disposant de plusieurs adresses ou d'un grand nombre d'essais successifs. Une file bornée seule ne prouverait pas l'équité.
- Les lectures de compte et l'admission de transactions rejouent le mempool sous verrou, avec vérifications cryptographiques. Le travail proposé dans l'issue #7 reste nécessaire, en conservant des vérifications d'état correctes après réorganisation.
- Ce transport TCP n'apporte ni DHT Kademlia, ni authentification des pairs, ni chiffrement des échanges, ni traversée générale des pare-feux. Une migration vers libp2p demanderait un choix de dépendances, un protocole et des tests distincts; elle ne résout pas à elle seule les invariants d'inventaire ou de consensus.

La prochaine preuve attendue est une CI rattachée au commit contenant ce correctif, suivie d'une campagne locale mesurant les ressources avec plusieurs pairs défaillants. Aucun résultat de ce document ne permet de présenter le réseau comme sûr pour des fonds réels.
