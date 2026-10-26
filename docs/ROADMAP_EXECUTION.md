# AURORA — état d’exécution et travaux restants

**Branche :** `feature/emission-schedule-v0`. **Cible :** mécanisme blockchain expérimental testnet, sans portefeuille, explorateur ni site de présentation dans cette étape. Toute la chaîne reste non auditée et impropre aux fonds réels.

## Réalisé et couvert par tests

- Encodage canonique des transactions/blocs, signatures Ed25519 vérifiées, nonce, solde et lot atomique.
- État par branche, validation de blocs, horodatage médian, retarget déterministe borné, travail cumulé u320 et réorganisation selon le travail.
- Émission `fast_start_500`, partage trésorerie/mineur et frais, plafond strict; identité mineur engagée et récompenses réorganisées avec l’état.
- Snapshot `AURORSNP` v3 avec toutes les branches et paramètres de difficulté/récompense; checksum, rejeu/revalidation et choix du dernier snapshot par travail cumulé (même si la chaîne gagnante est plus courte).
- Commande de minage testnet avec identité publique en argument, sauvegarde à chaque bloc et test de reprise après rechargement.
- TCP expérimental push/pull vers pairs statiques : sync paginée, réception et sauvegarde de blocs, propagation explicite par `mine --peer`, handshake d’empreinte consensus, découverte opt-in d’un saut, huit sessions concurrentes et quotas cumulés par session.
- `scripts/testnet_smoke.py` orchestre des processus séparés, pousse trois blocs, redémarre un pair pour tester le chargement du snapshot puis synchronise un répertoire neuf et vérifie la convergence; lancé par la CI.
- Tests réseau négatifs : blocs invalides, handshake incompatible, inventaires/frame malformés ou surdimensionnés, limites de session, découverte bornée et erreurs transitoires.
- CI GitHub Actions verte pour le lot mineur/snapshots (run 187, commit `983d56d16d572d4dfd988563ffdf0418d7c7261d`). CI GitHub Actions verte sur le lot courant (run 245, commit `c18186db1dc7a00fbed6bb22feb89e7d191bdf12`), couvrant P2P, limite d’octets de bloc et reprise par travail cumulé.

## Reste pour fermer une version de testnet reproductible

1. **Difficulté/temps** : vecteurs de référence indépendants, simulation de dérive temporelle, rafales, longues pauses et manipulation du retarget; décider une borne future de timestamp.
2. **Réseau** : le scénario multi-processus push/pull et les limites de ressources sont désormais vérifiés en CI. Restent une reconnexion continue, propagation multi-sauts spontanée, campagne de tests de coupure/réprise et fuzzing des trames; la topologie actuelle reste amorcée par adresses statiques, avec découverte opt-in d’un seul saut.
3. **Limites de bloc** : limite consensus de 4 MiB maintenant en place; mesurer le coût CPU/mémoire et tester les frontières de taille dans consensus, décodage, stockage et transport.
4. **Stockage** : injecter interruptions au moment d’écriture, disque plein, snapshots concurrents, restauration indépendante et migration de format; le snapshot actuel n’est pas une base transactionnelle.
5. **Testnet reproductible** : le smoke test à deux processus est en CI; ajouter scénarios multi-hôtes prolongés, synchronisation après redémarrage, forks concurrents et réorganisations observées entre processus.
6. **Robustesse** : fuzzing du décodage et de l’application de blocs, analyse de dépendances, revue humaine du code et revue indépendante du protocole.

## Étape suivante

La CI doit maintenant valider le transport TCP et son smoke test multi-processus; poursuivre les scénarios de coupure, redémarrage, réorganisation et messages hostiles. Le réseau fourni fonctionne pour une testnet expérimentale à pairs connus, sans promesse de disponibilité continue. La revue indépendante reste nécessaire avant toute finalisation publique ou tout usage avec valeur.
