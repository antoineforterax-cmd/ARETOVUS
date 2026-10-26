# AURORA — état d’exécution et travaux restants

**Branche :** `feature/emission-schedule-v0`. **Cible :** mécanisme blockchain expérimental testnet, sans portefeuille, explorateur ni site de présentation dans cette étape. Toute la chaîne reste non auditée et impropre aux fonds réels.

## Réalisé et couvert par tests

- Encodage canonique des transactions/blocs, signatures Ed25519 vérifiées, nonce, solde et lot atomique.
- État par branche, validation de blocs, horodatage médian, retarget déterministe borné, travail cumulé u320 et réorganisation selon le travail.
- Émission lissée `proportional-remaining-1e6-v1`, plafond brut strict de 500 M AUR, genèse sans préminage, répartition trésorerie 5 % / mineur 95 % et frais 50 % mineur / 50 % brûlés; réorganisations recalculent l’état.
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

## Après l’évaluation testnet

Le réseau expérimental fonctionne pour une testnet supervisée à pairs connus; les scénarios prolongés, le retarget indépendant, le fuzzing, les coupures disque et une revue indépendante restent avant tout usage public.

Ordre recommandé après ce jalon :
1. Évaluer le testnet à plusieurs nœuds et régler les erreurs de disponibilité, convergence, difficulté et restauration.
2. Simuler l’émission proportionnelle sur la durée et faire relire indépendamment la formule et ses incitations. La CI teste l’arithmétique, pas sa valeur économique.
3. Définir comment la trésorerie `[0xA0; 32]` peut dépenser : le compte actuel n’est pas un smart contract et ne porte pas de gouvernance ou de politique d’audit.
4. Concevoir les gardiens/time-locks avec les formats de clés et le portefeuille, puis tester récupération, révocation, délais et attaques de gardiens avant de les ajouter au consensus.
5. Traiter le recyclage des comptes inactifs comme une décision de propriété distincte : aucune saisie automatique n’est codée, et son seuil, les preuves d’inactivité, l’appel et les recours doivent être spécifiés avant prototypage.
6. Après stabilisation et revue indépendante du protocole, commencer le portefeuille, l’explorateur et le site explicatif. La vente ou la cotation vient bien plus tard, après sécurité, conformité et gouvernance du projet.
