# AURORA — feuille de route d'exécution

**Version de travail : 0.1 — 25 octobre 2026**  
**Statut : plan proposé pour le prototype, pas une validation du protocole.**  
**Règle : aucune étape n'est considérée terminée sans preuve vérifiable (tests, CI, revue et artefacts reproductibles).**

## Direction générale

Construire d'abord une chaîne expérimentale minimale, déterministe et testable. Ne pas tenter de livrer en même temps une crypto complète, une confidentialité avancée, une récupération sociale, une extension navigateur et un réseau public. Les fonctionnalités non essentielles restent des pistes de recherche jusqu'à ce que le cœur puisse résister à des tests adversariaux.

### Principes non négociables

- Aucun fonds réel, aucune promesse de valeur, aucun mainnet pendant le prototype.
- Cryptographie et primitives réseau issues de bibliothèques maintenues ; jamais de cryptographie maison.
- Les règles de consensus doivent être déterministes, explicites, versionnées et couvertes par des vecteurs de test.
- Les tests, Clippy et rustfmt doivent passer en CI avant de considérer une étape logicielle comme intégrée.
- Une CI verte ne prouve ni la sécurité ni l'absence de vulnérabilités ; revue indépendante et audit restent nécessaires.
- Les décisions économiques et protocolaires sont enregistrées comme PROPOSÉES jusqu'à confirmation et résultats suffisants.

## État observé du dépôt au début de ce plan

- Workspace Rust avec crates `aurora-core`, `aurora-consensus`, `aurora-node` et `aurora-wallet`.
- `aurora-core` contient déjà un type de montant, un encodage canonique expérimental, SHA-256 avec séparation de domaines, des identifiants de réseau et un modèle de transaction par comptes.
- Le registre expérimental vérifie maintenant Ed25519 strictement via `ed25519-dalek`; `sender` est la clé publique brute et `auth_data` la signature de 64 octets.
- L'empreinte canonique `signing_digest` couvre les champs hors `auth_data`, avec un domaine distinct du txid. Un profil de format est documenté; des vecteurs indépendants et la confirmation CI restent requis.
- Des blocs de comptes dédiés valident l’engagement, le réseau, le PoW et appliquent le lot Ed25519 atomiquement au registre; les frais sont retournés et aucune récompense n’est créée par ce profil. Le sélecteur conserve un instantané d’état pour chaque branche connue et réorganise aussi l’état canonique lorsque la branche gagnante change.
- Le registre suit aussi le cumul des unités émises et refuse les crédits d'émission dépassant le plafond global du type (`Amount::MAX`). Ce contrôle local ne remplace pas encore une comptabilité complète des récompenses, frais brûlés et réorganisations de chaîne.
- Un sélecteur en mémoire accepte des branches PoW concurrentes à difficulté fixe, choisit la plus longue et utilise le hash de sommet comme départage déterministe. Il ne valide pas l'état et n'est pas une règle de consensus à difficulté variable.
- La simulation d'émission existe dans `scripts/emission_sim.py`, mais ses sorties doivent être produites et revues de manière reproductible avant de figer une courbe.
- La CI a précédemment échoué sur rustfmt. Des corrections de formatage et un correctif aux tests du registre ont été poussés, mais aucune exécution CI confirmant le commit courant n'a été récupérée lors de cette vérification. Clippy, compilation et tests restent donc à confirmer.
- Le code actuel est expérimental ; aucune sécurité de production n'est revendiquée.

## Décisions de travail

| Sujet | Base de travail | Statut |
|---|---|---|
| Langage | Rust stable, version épinglée en CI | PROPOSÉ |
| Modèle | Comptes avec solde et nonce | PROPOSÉ |
| Précision | 8 décimales, unités entières | PROPOSÉ |
| Plafond brut | 500 000 000 AUR ; aucune émission au-delà, même après burn | PROPOSÉ |
| Préminage | Aucun | PROPOSÉ |
| Cible de bloc | 120 secondes en moyenne | PROPOSÉ |
| Consensus | Proof of Work SHA-256 | À SPÉCIFIER / TESTER |
| Difficulté | ASERT candidate, paramètres et vecteurs de référence obligatoires | À TESTER |
| Frais | 50 % mineur / 50 % burn, arrondis en unités minimales définis explicitement | À SIMULER |
| Trésorerie | 5 % de la récompense brute, incluse dans la récompense et non ajoutée au-dessus | À SIMULER ; gouvernance BLOQUANTE |
| Courbe d'émission | Scénarios 50 / 100 / 500 AUR, paliers inférieurs puis queue finale | À SIMULER |
| Signatures | Comparer bibliothèques éprouvées, format et coûts avant choix | BLOQUANT avant transactions signées |
| Post-quantique | Étude et format versionné ; aucune promesse de résistance quantique en v1 | RECHERCHE |
| Récupération | Facultative, consentie par le propriétaire, jamais par simple inactivité | HORS MVP ; modèle de menace requis |
| Confidentialité forte | Pas dans le premier MVP ; analyser impacts légaux, de taille et de vérification | HORS MVP |

## Feuille de route et critères de sortie

### Phase 0 — Contrôle de l'état et reproductibilité

**Actions**
1. Obtenir une CI verte pour le commit courant : formatage, Clippy, compilation, tests.
2. Vérifier les constantes monétaires et la documentation pour supprimer les contradictions.
3. Exécuter le simulateur d'émission, vérifier les invariants et archiver les résultats.
4. Ajouter des tests automatisés pour la simulation et documenter la commande exacte.

**Sortie exigée**
- CI complète verte et lien vers l'exécution.
- Rapport comparatif de l'émission comprenant durée théorique, blocs par palier, offre brute, trésorerie et part mineur.
- Aucun résultat présenté comme mesuré s'il est seulement théorique.

### Phase 1 — Spécification du protocole minimal

**Actions**
1. Définir le format de transaction et d'en-tête de bloc, les versions et les limites de taille.
2. Choisir l'encodage de signatures et l'algorithme à partir de bibliothèques maintenues ; valider l'empreinte de signature par spécification et vecteurs de référence.
3. Définir précisément l'engagement sur les transactions : arbre de Merkle, ordre des feuilles, règles pour un nombre impair, preuve et vecteurs.
4. Définir l'identifiant de chaîne/réseau, la sérialisation canonique et les erreurs de validation.
5. Établir un modèle de menace : fausse signature, rejeu inter-réseaux, nonce, double dépense, overflow, DoS, réorganisation, spam et compromission de clés.

**Sortie exigée**
- Spécification versionnée avec exemples d'octets exacts et vecteurs de référence.
- Aucune règle critique laissée à l'interprétation du code.

### Phase 2 — Registre d'état et transactions valides

**État actuel : partiellement implémenté, non terminé.** Le registre en mémoire vérifie les signatures Ed25519, le réseau, les nonces, les soldes et l'émission atomiquement. Les règles complètes d'émission, de frais, de bloc et de réorganisation manquent encore.

**Actions**
1. [x] Prototype en mémoire d'un état de comptes déterministe : solde et nonce.
2. [x] Contrôles préalables du réseau, nonce exact, signature Ed25519, solde pour montant + frais et arithmétique vérifiée.
3. [x] Rejet sans mutation partielle pour les cas déjà couverts par les tests unitaires.
4. [ ] Définir le traitement des transactions concurrentes d'un même compte et les règles du mempool.
5. [x] Intégrer la vérification Ed25519 avec `ed25519-dalek` et documenter le profil expérimental signé.
6. [ ] Définir frais minimums, comptabilité du burn, trésorerie et récompense de bloc; intégrer la reconstruction d’état au sélecteur de branches.
7. [ ] Ajouter tests de propriétés/fuzzing et confirmer l'ensemble par CI verte.

**Sortie exigée**
- Tests pour solde insuffisant, double dépense, rejeu, mauvais nonce, mauvaise signature, frais, overflow/underflow, état inchangé après rejet et invariant de conservation.
- Tests de propriétés/fuzzing là où ils sont pratiques.

### Phase 3 — Politique monétaire et émission

**Actions**
1. Corriger et tester le simulateur, sans l'utiliser comme code de consensus.
2. Comparer les scénarios de départ 50, 100 et 500 AUR, y compris les seuils, la queue à 1 AUR et une éventuelle décroissance vers zéro.
3. Tester les effets de variation du temps de bloc, de la puissance de calcul et de l'arrivée de mineurs.
4. Décider si la règle est uniquement basée sur la hauteur, ou sur la hauteur et l'offre créée. Éviter une formule hybride ambiguë.
5. Spécifier la règle d'arrondi de la trésorerie et des frais en unités minimales.
6. Vérifier que le plafond brut de 500 millions ne peut être dépassé, y compris en cas de réorganisation.

**Sortie exigée**
- Résultats reproductibles, hypothèses affichées et courbe complète définie sans interpolation implicite.
- Confirmation explicite du porteur du projet avant de coder la courbe dans le consensus.

### Phase 4 — Consensus PoW et difficulté

**Actions**
1. Spécifier les champs de l'en-tête et exactement quels octets sont hachés.
2. Définir la conversion cible/difficulté, le calcul du travail, les limites et la comparaison des chaînes.
3. Évaluer ASERT contre un jeu de vecteurs de référence, avec ancre, paramètres, arithmétique entière et bornes spécifiés.
4. Définir les règles de réorganisation, les égalités, les confirmations et le comportement après redémarrage.
5. Tester timestamps extrêmes, longues périodes sans blocs, arrivées en rafale, blocs invalides et réorganisations profondes.
6. Ne pas rendre la difficulté « plus dure que Bitcoin » comme objectif en soi : la difficulté doit suivre le hashrate et la sécurité réelle du réseau, pas un chiffre symbolique.

**Sortie exigée**
- Vecteurs identiques pour tous les nœuds, tests de reorg, tests de limites et spécification de chaîne valide avec travail cumulé maximal.

### Phase 5 — Stockage et nœud local

**Actions**
1. Ajouter une persistance avec format versionné et migrations explicites.
2. Reconstruire l'état depuis la chaîne et vérifier le résultat contre l'état stocké.
3. Gérer arrêt brutal, corruption, disque plein, base incohérente et reprise.
4. Fournir des commandes locales pour créer un réseau de développement, miner des blocs et envoyer des transactions de test.

**Sortie exigée**
- Tests de redémarrage et de reconstruction, fixtures de chaînes, erreurs de stockage traitées sans corruption silencieuse.

### Phase 6 — Réseau P2P

**Actions**
1. Définir le protocole de messages versionné et les limites strictes de taille/temps.
2. Mettre en place découverte de pairs, connexion, propagation des transactions et des blocs, synchronisation et reprise après déconnexion.
3. Limiter les connexions, la bande passante, les files d'attente, les requêtes et les annonces répétées.
4. Protéger le nœud contre messages malformés, flood, pairs menteurs, identifiants répétés et consommation mémoire excessive.
5. Ajouter métriques, journaux utiles et mode de diagnostic sans exposer de secrets.

**Sortie exigée**
- Tests avec pairs malveillants/simulateurs, reconnexion, synchronisation interrompue et plusieurs nœuds aux états différents.

### Phase 7 — Portefeuille et expérience utilisateur

**Actions**
1. Séparer le portefeuille du nœud ; les clés privées ne sont jamais envoyées au nœud.
2. Générer et sauvegarder les clés, afficher les avertissements de sauvegarde et tester la restauration.
3. Définir le format d'adresse avec identifiant réseau et somme de contrôle.
4. Avant signature, afficher réseau, destinataire, montant, frais et total ; éviter les erreurs de presse-papiers et de mauvais réseau.
5. Évaluer la compatibilité avec portefeuille matériel uniquement après stabilisation du format de signature.
6. Étudier la récupération volontaire comme protocole distinct : consentement, délais, révocation, collusion, perte de gardiens et attaques de récupération.

**Sortie exigée**
- Tests de restauration, séparation des clés, adresses malformées, mauvais réseau et confirmation de transaction lisible.
- Pas de récupération automatique ni de dépositaire caché.

### Phase 8 — Testnet fermée puis publique

**Actions**
1. Lancer d'abord un réseau local reproductible, puis une testnet fermée entre machines contrôlées.
2. Publier les instructions de démarrage, les paramètres de chaîne et la façon de réinitialiser les données de test.
3. Faire des tests de charge, partition réseau, perte de pairs, changement de difficulté et réorganisation.
4. Organiser une revue indépendante du code et du modèle de menace ; corriger les problèmes avant d'ouvrir davantage.
5. Publier les limites connues et un canal de signalement de vulnérabilités.

**Sortie exigée**
- Nœuds indépendants convergents, tests reproductibles, aucun bug critique ouvert et procédures d'incident écrites.
- La testnet n'est pas une preuve de sécurité économique ou cryptographique.

### Phase 9 — Recherches futures, hors MVP

- Signatures post-quantiques : taille des signatures, coût de vérification, compatibilité et plan de migration.
- Confidentialité : analyse du modèle de menace, fuite de métadonnées, coûts de vérification et exigences légales.
- Extension de navigateur anti-traqueurs : projet séparé, ne pas mélanger ses privilèges au portefeuille.
- Pénalités anti-baleines : ne pas implémenter sans définition mathématique et analyse des contournements (fractionnement de comptes, entités liées, liquidité et équité).
- Récupération de clés : prototype séparé, opt-in et auditable.
- Gouvernance de trésorerie : multisignature, délais, transparence, limites de dépenses et protection contre la capture.

## Tests à automatiser en continu

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo build --workspace --locked` une fois le fichier de verrouillage commité
- Tests Python du simulateur et vérification des invariants monétaires
- Fuzzing du décodeur et des messages réseau
- Vecteurs de référence pour hash, empreinte de signature, transaction, bloc, difficulté et émission
- Tests de propriétés : aucun dépassement de plafond, aucun overflow silencieux, même entrée => même résultat, rejet sans mutation d'état

## Politique de livraison

Chaque changement doit être petit, relu et lié à une étape. Une étape ne passe à **TERMINÉE** qu'après vérification des sorties exigées. Tout défaut découvert doit être documenté et corrigé avant d'ajouter une couche supérieure. Si un choix nécessite une décision du propriétaire, conserver la proposition comme telle et poursuivre sur les parties réversibles.

## Prochaine action concrète

Stabiliser la CI et le noyau de types/transactions, vérifier le plafond monétaire dans tout le dépôt, puis tester le simulateur d'émission. Ne pas démarrer le P2P ni déclarer la testnet prête avant que l'application d'état, les signatures, les règles de blocs et la sélection de chaîne soient spécifiées et testées.
