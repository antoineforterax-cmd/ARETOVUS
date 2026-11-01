# ARETOVUS — Architecture présente et options futures

Cette description concerne le testnet en développement, pas le code historique de `main`. Les correctifs récents sur les connexions P2P, le cache du mempool, les réorganisations et la validation des adresses sont répartis entre les PR #16 à #22, non fusionnées au 29 octobre 2026.

## A. Flux actuel (testnet)
`wallet CLI -> transaction signée ML-DSA-65 -> node/P2P TCP -> validation core/consensus -> mempool -> PoW/bloc -> état comptes+nonces -> stockage`.

Les couches ont des responsabilités distinctes :
- **core** : structures, sérialisation canonique, hashes et vérification de signatures ;
- **consensus** : règles d'admission, état, émission, difficulté, chaîne et réorganisations ;
- **node** : réseau, synchronisation, mempool, stockage et exécution ;
- **wallet** : clés, signature, sauvegarde et consultation.

La séparation logique ne constitue pas un audit de découplage des dépendances source.

## B. Architecture cible par domaines
1. **Crypto et consensus** : primitives versionnées, profils de transaction explicites, tests de vecteurs, aucune activation implicite.
2. **État et stockage** : engagements de blocs, rejouabilité déterministe, reprise et migrations contrôlées.
3. **Réseau** : couche de transport abstraite, identification du profil, découverte et politique de relais séparées ; TCP reste le transport de référence jusqu'à preuve contraire.
4. **Wallet** : gestion des secrets, clés de consultation éventuelles, sauvegardes, compatibilité appareils et UX.
5. **Observabilité** : métriques locales, logs sans secrets, mesure des ressources et tests de convergence.
6. **Gouvernance** : spécifications versionnées, activation des règles, traçabilité des changements.

## C. Confidentialité — programme de recherche, pas plan de fusion immédiat
**Modèle de menace à écrire** : observateur réseau, pair malveillant, analyse de graphe, mineur, opérateur de nœud, compromission du wallet, adversaire quantique futur.

**Couche transport** : évaluer Tor et I2P comme transports optionnels ou natifs ; mesurer l'impact sur latence, disponibilité, découverte et résistance aux attaques Sybil. Tor ne masque pas automatiquement les corrélations de transactions. Dandelion++ est une politique de diffusion à évaluer séparément ; son ajout à un transport onion ne garantit pas un gain.

**Couche transaction** : adresses furtives, confidentialité des montants, confidentialité de l'expéditeur et résistance à l'analyse doivent être étudiées ensemble. Les engagements de Pedersen seuls ne prouvent ni l'absence de création monétaire ni la positivité des montants : preuves de plage, conservation et prévention des doubles dépenses sont nécessaires.

**Changement structurel** : le modèle comptes+soldes+nonces publics ne permet pas la confidentialité complète par simple ajout d'un champ. Comparer des modèles notes/UTXO à engagements, nullifiers et preuves, ou un état confidentiel alternatif. Étudier coût de preuve, vérification, stockage, auditabilité de l'offre, réorganisations et migration.

**Post-quantique** : ML-DSA-65 protège la signature selon son hypothèse, pas l'ensemble du protocole. Les signatures en anneau et engagements classiques basés sur courbes elliptiques ne deviennent pas post-quantiques par association à ML-DSA. Exiger une analyse primitive par primitive.

**Trésorerie et audit** : définir comment vérifier l'offre, les récompenses et les dépenses de trésorerie si les montants sont cachés ; envisager des preuves publiques ou une divulgation volontaire contrôlée sans présumer de leur faisabilité.

## D. Dépendances avant implémentation de confidentialité
Modèle de menace -> comparaison cryptographique -> preuve de concept isolée -> benchmarks -> revue spécialisée -> proposition de protocole et migration -> tests adversariaux -> éventuel testnet incompatible dédié.

## E. Non-objectifs
Aucune promesse d'anonymat absolu, d'impossibilité de traçage, d'anti-51 % garanti ou de performance sans mesures. Aucune activation de confidentialité par simple changement de transport.


## F. Matrice des fuites de confidentialité à prévenir (priorité produit)

| Surface | Risque | Piste de conception | Limite / test à prévoir |
|---|---|---|---|
| IP et empreinte réseau | Association de pairs, de nœuds et de transactions à une adresse réseau | Transports préservant la confidentialité (Tor/I2P), isolation des circuits, minimisation des métadonnées, chiffrement authentifié | Chiffrement TCP seul ne masque pas les IP ; mesurer fuites DNS, bootstrap, connexions directes, corrélations et disponibilité |
| Réutilisation d'adresse | Liaison de paiements et construction de graphes | Réception avec identifiants/adresses à usage unique ou mécanisme équivalent, gestion des clés de consultation | Une adresse furtive ne masque pas automatiquement expéditeur, montant, IP ou liens issus d'autres données |
| Montants publics | Déduction des habitudes, revenus et contreparties | Étudier engagements de valeurs, preuves de plage, conservation, prévention des doubles dépenses, modèle notes/nullifiers | Engagements de Pedersen classiques non automatiquement post-quantiques ; ne pas assimiler engagement à preuve complète |
| Horodatage et relais | Inférence probabiliste de l'émetteur par observation des premières diffusions | Étudier Dandelion++ ou autres politiques de relais, délais et protections contre corrélation | Pas d'anonymat absolu ; efficacité dépend de la topologie, des observateurs, de la gestion des nœuds et des mesures |
| Wallet et récupération | Métadonnées révélées par synchronisation, sauvegarde ou gardiens | Vérifications locales, minimisation des données, récupération à divulgation limitée | Tester collusion des gardiens, vol d'appareil, restauration et fuites de métadonnées |

### Contraintes transversales
1. Concevoir une politique explicite des métadonnées : données strictement nécessaires, durée de conservation, accès, journalisation et divulgation volontaire.
2. Séparer confidentialité **réseau**, **transactions**, **montants**, **wallet** et **récupération** ; aucune protection isolée ne suffit.
3. Conserver une validation indépendante de l'offre, des frais, des récompenses et des doubles dépenses.
4. Mesurer l'effet sur latence, propagation des blocs, bande passante, RAM, CPU, stockage et décentralisation.
5. Évaluer les garanties post-quantiques de chaque primitive et ne pas annoncer de confidentialité absolue.
6. Toute modification de l'état public actuel doit passer par un prototype isolé, une spécification versionnée et des tests adversariaux avant une éventuelle intégration.
