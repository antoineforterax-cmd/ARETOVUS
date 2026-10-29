> Périmètre révisé le 27 octobre 2026 : explorateur, site, vente simulée, interfaces graphiques et mobile sont reportés aux repreneurs. Les anciennes étapes applicatives ci-dessous sont conservées comme historique, pas comme travaux en cours.

## Situation au 29 octobre 2026

Le travail actif est réparti entre la branche `feature/emission-schedule-v0` et plusieurs PR en brouillon. Le README de `main` explique désormais cette distinction. La release du 27 octobre reste un instantané expérimental, pas une version stable ni la somme des modifications récentes.

**Fait et testé dans les branches concernées :** signatures ML-DSA-65 v3, consensus et économie de testnet, stockage v6, P2P TCP, wallet CLI chiffré, tests multi-nœuds ; stockage et rechargement de 100 003 blocs ; essais ciblés des auto-transferts (#13), du retarget (#14), de l'admission P2P (#16), du cache cryptographique (#17-#18), du fuzzing des comptes (#19), des réorganisations (#20-#21) et du décodage d'adresses (#22). Les preuves et limites figurent dans chaque PR.

**À terminer avant un testnet durable :** intégrer les PR après revue, rejouer la CI sur l'ensemble, terminer les scénarios d'horloges divergentes (#3), de saturation après handshake et sous NAT (#4), et de synchronisation P2P à grande échelle (#5). Publier le bilan final de la campagne 72 h lancée le 28 octobre, puis éprouver plusieurs machines réelles, les partitions et les pannes de stockage. Aucun audit humain indépendant n'a été réalisé.

**Études à poursuivre séparément :** confidentialité des transactions et des métadonnées, récupération volontaire des accès, protections anti-arnaque, gouvernance de trésorerie et compatibilité post-quantique globale. Ces objectifs ne sont pas des fonctions actives du testnet.

**Applications reportées :** explorateur, site complet, wallet graphique et mobile. Ne pas les présenter comme disponibles.

Les sections suivantes conservent leurs dates et résultats d'origine ; les mentions de stockage v5 ou d'absence de release décrivent des étapes antérieures et ne remplacent pas les preuves v6 et la publication du 27 octobre.

---


# AURORA — état vérifié et chemin vers une version finale

**Branche de travail :** `feature/emission-schedule-v0`.  
**État :** testnet expérimental supervisé. La PR reste non fusionnée. Aucun fonds réels; aucune revendication de sécurité mainnet.

## 1. Éprouver le réseau et le stockage

### Déjà couvert par les contrôles automatisés

- Validation de transactions ML-DSA-65 v3, blocs, retarget borné, MTP, travail cumulé et réorganisations.
- Stockage v5 par blocs immuables et index checksummés avec trois points de reprise; relecture/revalidation des branches, retour à un index valide antérieur si le dernier est corrompu, lecture des snapshots v4 et restauration d’une copie dans un autre dossier. Les commandes `mine`, `sync` et `serve` prennent un verrou exclusif sur leur dossier de données; le smoke test vérifie le refus d’un second écrivain.
- Décodage hostile : tailles excessives, trames/inventaires malformés, handshake incompatible, bloc rejeté et limites de session. La lecture et l’écriture complètes d’une trame ont chacune un délai total; les réponses volumineuses sont écrites après libération des verrous de chaîne et de mempool. Une page n’est publiée en mémoire qu’après validation et sauvegarde complètes.
- Scénario multi-processus : le smoke test mine et propage, inclut une transaction wallet, vérifie son statut et sa preuve Merkle, passe la hauteur 20 à travers deux fenêtres de retarget, redémarre un pair et synchronise les blocs manquants. Un serveur déjà lancé rattrape automatiquement un pair à la hauteur 21 puis un nouveau bloc à la hauteur 22. Le mempool est récupéré par pages depuis les pairs configurés; les transactions sont revalidées avant admission locale, relayées et minées par un pair distinct dans le smoke test.
- Essai local à trois nœuds : 36 blocs propagés en 490,2 secondes avec quatre redémarrages d’un pair et restauration vérifiée d’une copie complète; [preuve CI](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40361895189).
- Reprise après terminaison forcée entre des sauvegardes; essai d’échec d’écriture simulé par limite de taille de fichier sur Linux.
- Test reproductible déterministe d’entrées arbitraires pour les décodeurs de transactions, blocs, en-têtes, inventaires et listes de pairs.
- Trois cibles libFuzzer pour les décodeurs canoniques, les preuves Merkle et les charges P2P; job GitHub Actions séparé. Le [rapport](TESTNET_VALIDATION_REPORT.md) donne les nombres d’entrées et les limites.

### À compléter avant de prétendre à une robustesse élevée

- Campagnes fuzz longues et rapport de couverture; le job CI court ne remplace pas une campagne de plusieurs heures/jours.
- Le test de déconnexion au milieu d’une page vérifie désormais l’absence de publication partielle. Restent les [horodatages futurs](https://github.com/antoineforterax-cmd/AURORA/issues/3), la [saturation simultanée des workers](https://github.com/antoineforterax-cmd/AURORA/issues/4), les inventaires menteurs et les partitions répétées.
- Une copie complète de dossier est restaurée par test automatisé, et la lecture v4 puis écriture v5 est couverte. Il reste à éprouver une sauvegarde hors machine réelle, un disque effectivement plein et une panne électrique. La limite de taille du fichier reproduit une erreur d’écriture; elle ne simule pas un volume saturé.
- Le stockage et le fil P2P n’ont plus de seuil logique à 100 000 blocs, mais le franchissement réel de cette borne et la croissance mémoire/CPU restent à prouver avant un testnet durable. Il faut également éprouver des réorganisations concurrentes répétées et une synchronisation pendant des jours sur plusieurs machines réelles; les 490,2 secondes de rattrapage local restent un essai borné.
- Vérification indépendante du retarget sous dérive d’horloge, longues pauses et rafales.

## 2. Revue indépendante — reportée pour le développement expérimental

Le dossier [INDEPENDENT_REVIEW.md](INDEPENDENT_REVIEW.md) décrit le périmètre, les commandes et les limites connues. Une [contre-revue automatisée](AI_COUNTER_REVIEW.md) a relevé des problèmes vérifiables et conduit à corriger l'indexation des lectures réseau. Elle ne remplace pas une revue humaine indépendante. Aucune revue humaine indépendante n’a été réalisée. Les contrôles CI, ce travail d’implémentation et les outils automatisés ne comptent pas comme une telle revue.

Le 26 octobre 2026, le porteur a choisi de continuer le développement expérimental sans revue externe immédiate. Aucun reviewer indépendant n’est actuellement désigné et aucune demande n’a été envoyée. Cette décision permet de poursuivre les outils et essais locaux; elle ne donne aucune assurance de sécurité. Avant toute revendication mainnet ou usage de fonds réels, il faut un auditeur humain indépendant, un commit épinglé, un rapport écrit, des corrections et de nouveaux tests.

## 3. Décisions de protocole et économie

Les paramètres économiques demandés par le porteur sont codés : plafond brut de 500 millions AUR, aucune préémission, émission proportionnelle décroissante, 5 % crédités à l’adresse de trésorerie de test, 95 % au mineur, frais partagés 50/50 et burn non réutilisable sous le plafond.

Les décisions de périmètre sont détaillées dans [TESTNET_DECISIONS.md](TESTNET_DECISIONS.md) :

- L’adresse de trésorerie reçoit la part prévue, mais il n’existe ni contrat, ni gouvernance, ni règles de dépense/audit. Ne pas présenter cette adresse comme une trésorerie autonome opérationnelle avant qu’une politique de contrôle et d’utilisation soit décidée puis revue.
- Le protocole actuel ne saisit et ne recycle pas les soldes inactifs. Toute récupération d’actifs change les droits de propriété; elle exige une décision explicite sur le seuil, les preuves, le recours et les protections contre le vol de clés.
- DID, gardiens et récupération restent au stade de conception hors consensus dans [DID_RECOVERY_DESIGN.md](DID_RECOVERY_DESIGN.md). Ils ne peuvent être ajoutés à l’état des comptes qu’après conception détaillée, revue et décision du porteur.

## 4. Stabiliser le testnet

La configuration locale reproductible et son guide sont disponibles dans [TESTNET_GUIDE.md](TESTNET_GUIDE.md). Les procédures de vérification, sauvegarde, restauration, mise à jour et incident sont dans [TESTNET_OPERATIONS.md](TESTNET_OPERATIONS.md). Les paramètres de preuve de travail sont délibérément faibles et la topologie demande des pairs connus. La CI valide un scénario local court et un essai séparé à trois nœuds de 490,2 secondes, pas un réseau public exploité dans la durée. Des archives testnet privées Windows, Linux et macOS sont disponibles en artefacts CI pendant 14 jours; elles ne sont ni signées ni publiées comme releases.

Avant de déclarer un testnet public stable, il faut choisir et héberger des nœuds semences, réaliser une campagne multi-hôtes prolongée, surveiller la disponibilité/convergence, geler une version protocolaire, et documenter les mises à jour, sauvegardes, restauration et signalement d’incident. Aucun hôte public ni opérateur externe n’est fourni dans ce dépôt.

## 5. Applications — après les portes précédentes

Ces éléments restent des prototypes de testnet tant que les essais prolongés, les décisions de protocole et la revue indépendante n’ont pas été achevés. Leur construction peut continuer, mais pas leur présentation comme produits sûrs pour des fonds réels :

1. **Portefeuille quotidien** : les nouveaux fichiers de clés du CLI testnet sont chiffrés par défaut (Argon2id/XChaCha20-Poly1305), avec commandes de sauvegarde et restauration chiffrées, vérification de l’identité restaurée, refus d’écrasement et contrôle de l’empreinte consensus annoncée par le nœud. Restent des essais d’ergonomie, une gestion des secrets par plateforme et une revue indépendante avant toute utilisation de fonds réels.
2. **Explorateur** : indexeur et API en lecture seule avec affichage des blocs, transactions, preuves et réorganisations.
3. **Site de présentation** : but du projet, fonctionnement, limites, guide d’installation et risques.
4. **Vente/échange** : spécification séparée des rôles, garde des fonds, règlement et exigences juridiques applicables avant toute intégration financière.

## Critères de version finale

- Campagnes de tests prolongées et fuzzing avec résultats conservés.
- Revue indépendante terminée et constats critiques/élevés corrigés ou formellement acceptés par le porteur.
- Décisions documentées sur trésorerie, inactivité, DID/récupération et mises à niveau.
- Testnet multi-hôtes stable, version et procédures reproductibles publiées.
- Portefeuille, explorateur et site testés sur la version protocolaire gelée.
- Toute vente ou cotation fait l’objet d’une validation juridique et opérationnelle distincte.

La version actuelle ne satisfait pas encore ces critères.