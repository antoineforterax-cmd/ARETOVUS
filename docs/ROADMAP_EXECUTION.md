# AURORA — état d’exécution et travaux restants

**Branche :** `feature/emission-schedule-v0`. **Cible :** mécanisme blockchain expérimental testnet, sans portefeuille, explorateur ni site de présentation dans cette étape. Toute la chaîne reste non auditée et impropre aux fonds réels.

## Réalisé et couvert par tests

- Encodage canonique des transactions/blocs, signatures Ed25519 vérifiées, nonce, solde et lot atomique.
- État par branche, validation de blocs, horodatage médian, retarget déterministe borné, travail cumulé u320 et réorganisation selon le travail.
- Émission `fast_start_500`, partage trésorerie/mineur et frais, plafond strict; identité mineur engagée et récompenses réorganisées avec l’état.
- Snapshot `AURORSNP` v3 avec toutes les branches et paramètres de difficulté/récompense; checksum, rejeu et revalidation à la reprise.
- Commande de minage testnet avec identité publique en argument, sauvegarde à chaque bloc et test de reprise après rechargement.
- TCP expérimental push/pull vers pairs statiques : sync paginée, réception de blocs, propagation explicite par `mine --peer`, handshake d’empreinte consensus, limites de frames et revalidation des blocs reçus.
- CI GitHub Actions verte pour le lot mineur/snapshots (run 187, commit `983d56d16d572d4dfd988563ffdf0418d7c7261d`). CI GitHub Actions verte sur le lot P2P push/pull (run 215, commit `41f5b811f0d620a61ec1c741bdf012720a31cacd`).

## Reste pour fermer une version de testnet reproductible

1. **Difficulté/temps** : vecteurs de référence indépendants, simulation de dérive temporelle, rafales, longues pauses et manipulation du retarget; décider une borne future de timestamp.
2. **Réseau** : tests de protocole contre pairs malveillants, coupure/reprise, plafonds cumulés de ressources par session; ajouter découverte, reconnexion et gestion multi-pairs ou documenter précisément une topologie statique expérimentale.
3. **Limites de bloc** : limite consensus de 4 MiB maintenant en place; mesurer le coût CPU/mémoire et tester les frontières de taille dans consensus, décodage, stockage et transport.
4. **Stockage** : injecter interruptions au moment d’écriture, disque plein, snapshots concurrents, restauration indépendante et migration de format; le snapshot actuel n’est pas une base transactionnelle.
5. **Testnet reproductible** : script d’orchestration multi-nœuds, commande de vérification de solde/état, tests de divergence et de réorganisation entre processus.
6. **Robustesse** : fuzzing du décodage et de l’application de blocs, analyse de dépendances, revue humaine du code et revue indépendante du protocole.

## Étape suivante

Faire passer la CI du transport TCP ajouté, corriger toute régression, puis écrire/valider les scénarios multi-nœuds et les attaques réseau. La crypto sera considérée comme codée pour le testnet seulement lorsque ces critères centraux seront exécutables et que les limites restantes seront clairement signalées. La revue indépendante restera nécessaire avant toute finalisation ou usage public.
