# C1 — Triage du test d'admission P2P (2 novembre 2026)

> Statut : **non résolu**, PR #165 en brouillon. Tests exécutés exclusivement sur GitHub Actions Linux, en loopback sur un nœud ARETOVUS expérimental. Aucun vrai pair distant ni fonds réel.

## Identité des expériences

- Branche : `test/c1-hostile-admission-resource-bounds-20261102`.
- SHA testé : `bc48d5598078b651381b96f90bd56f471d04c2f4` ; parent `main@398cc258aebb928bcf0389eee279b361a5f52d1e`.
- [Run #41463438780](https://github.com/antoineforterax-cmd/ARETOVUS/actions/runs/41463438780), tentative 1 : échec du job `hostile-loopback` ; compilation réussie ; artefact `c1-hostile-admission-evidence` uploadé.
- Même run, tentative 2 : **échec reproduit**, même exception au même stade.
- [Run #41463509592](https://github.com/antoineforterax-cmd/ARETOVUS/actions/runs/41463509592) : échec indépendant, même stade.
- [CI générale #41463438715](https://github.com/antoineforterax-cmd/ARETOVUS/actions/runs/41463438715) : succès sur le même SHA. La CI générale ne remplace pas la campagne C1.

## Symptôme observé

Le sondage wallet initial fonctionne. Après la série de connexions avec préfaces refusées depuis `127.0.0.1`, la connexion de contrôle wallet reçoit `ConnectionResetError: [Errno 104] Connection reset by peer`, avant réception de son en-tête de réponse (`wallet_probe()`, appelé dans `after_malformed_handshakes`). Le journal ne démontre pas de crash du processus nœud. Aucun résultat de latence/RSS/CPU sous saturation post-handshake n'a été obtenu après ce stade.

## Hypothèse plausible, **pas encore prouvée**

`PeerLease::acquire()` limite à quatre connexions simultanées par IP (`MAX_ACTIVE_PER_IP=4`). Les sockets de test se ferment rapidement côté client mais leurs baux ne disparaissent que lorsque le dispatcher les détecte et détruit leurs sessions en attente ou actives. Un wallet soumis dans la même rafale peut donc être rejeté avant que ces baux soient libérés. Le serveur applique alors possiblement sa politique prévue, plutôt qu'un défaut inédit de consensus ou de parsing. Une seule machine et un seul IP ne permettent pas de conclure quant au NAT réel.

## Procédure de décision / critères de correction

1. Conserver cette preuve rouge inchangée et préciser les événements de libération des baux ; comparer les timestamps de fermeture, d'acceptation et de rejet. Ne pas assimiler une fermeture client à une libération synchronisée du quota serveur.
2. Tester séparément la disponibilité depuis une autre adresse source loopback autorisée, pour évaluer l'isolation entre IP.
3. Tester depuis la même IP *après* confirmation mesurée du retour sous le quota, avec une borne de récupération explicite. Ne pas supprimer l'assertion initiale sans documenter le comportement voulu sous NAT.
4. Garder le scénario original rouge, ou le transformer en cas négatif explicitement attendu, **et** ajouter une assertion positive de récupération. Ne pas relever arbitrairement les limites, désactiver des contrôles ou simuler un succès.
5. Rejouer le test avant/après correctif au SHA exact et conserver RAM/CPU du processus nœud, latence wallet, convergence d'un pair neuf et artefact JSON. Vérifier aussi les workflows P2P, Windows et smoke concernés.
6. Ne cocher C1 ni fusionner la PR #165 avant `completed/success` des contrôles pertinents et contre-revue des risques sous NAT/transport WAN. Une CI verte resterait une validation locale bornée, pas une preuve de résistance au DoS public.

## Portée et blocages

Aucune correction du code de production n'a été faite. La modification de scénario suggérée lors de la revue précédente n'a pas pu être appliquée. La résistance aux pairs WAN, aux attaques distribuées, aux connexions prolongées derrière un NAT et aux effets de concurrence sur différents OS reste non validée.

## Test d'isolation complémentaire — récupération séquentielle

La branche contient désormais `scripts/testnet_lease_recovery_probe.py` et le workflow `c1-sequential-lease-recovery`. Cette campagne n'exécute **jamais** de saturation simultanée : une seule connexion invalide locale est ouverte puis fermée avant chaque wallet.

- Source testée : `0581ab24d1f14275468b9eff970393401d1e875e`.
- [Run #41464758219](https://github.com/antoineforterax-cmd/ARETOVUS/actions/runs/41464758219) : **completed/success** ; artefact `c1-sequential-lease-recovery-evidence` présent.
- Huit cycles successifs sur `127.0.0.1` ; wallet répond à chaque premier essai, en environ **0,0202–0,0204 seconde**, sans retry ni exception. Baseline wallet : environ 0,0207 s.
- Le test C1 de concurrence `c1-hostile-admission` reste **failure** au même SHA. Les deux résultats ne sont pas contradictoires : la récupération séquentielle ne prouve pas une disponibilité après rafale de connexions ou sous NAT.

**Décision :** ne pas modifier le seuil de connexions ni cocher C1 ; instrumenter ensuite la concurrence/quota en gardant un contrôle depuis une autre adresse source loopback et une reprise mesurée depuis la même IP.
