# ARETOVUS V2 — architecture proposée pour le produit graphique

**Statut : plan d'implémentation, pas un produit fonctionnel ni une décision de changement de protocole.** Référence : [feuille de route V2 #178](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178). Le code du testnet ne doit pas être utilisé avec des fonds réels.

## Architecture du premier produit

```text
Interface graphique desktop (React/Vite, conteneur à choisir via ADR)
    |
    | IPC local / canal authentifié, schémas versionnés
    v
Pont de contrôle local Rust (lecture seule en premier)
    |    |             |
    |    |             +--> wallet Rust (clés chiffrées; signatures strictement locales)
    |    +--> indexeur lecture seule (données dérivées, reconstruction possible)
    +--> aretovus-node (statut, vérification, minage, connexion réseau)
                  |
                  +--> TCP P2P natif ARETOVUS + validation consensus
```

**La GUI ne parle pas directement TCP aux pairs depuis un navigateur.** Le nœud transporte les blocs et transactions en P2P, vérifie le consensus, alimente l'indexeur ; l'interface lit des états validés via son pont local. L'index n'a jamais autorité pour modifier la chaîne ou déclarer une transaction définitive.

**Politique des secrets :** aucune clé privée dans le JavaScript, le navigateur, le localStorage/sessionStorage, les messages de télémétrie, les URLs ni les logs. Le futur wallet graphique utilise les fonctions Rust déjà testées pour chiffrer, déchiffrer et signer localement, sous une action de l'utilisateur ; pas de passerelle RPC d'écriture exposée publiquement.

## Couche de statut read-only V1.1 réellement implémentée

Le fichier `scripts/aretovus_status_json.py` constitue le **tout premier adaptateur en lecture seule**, pas un service réseau. Exemple :

```sh
python3 scripts/aretovus_status_json.py \
  --node-binary ./target/debug/aretovus-node \
  --data-dir ./mon-noeud-testnet
```

Il exécute `aretovus-node testnet verify --data-dir ...` par `subprocess.run`, **sans shell et sans port TCP**, vérifie le format strict de la réponse du nœud et émet un seul objet JSON. Il limite le temps d'exécution, la taille maximale de sortie, le nombre de blocs cohérent et la longueur du travail de chaîne. En cas d'erreur, l'application reçoit un **code prédéfini** ; jamais le stderr ni les chemins du système. Le `work` affiché actuellement est une chaîne de diagnostic Debug, **pas** une représentation numérique garantie : définir un type d'API stable dans le nœud avant une GUI experte.

Le statut provient d'un **dossier de nœud existant et vérifié** ; il ne démarre pas de nœud, ne mine aucun bloc, ne synchronise aucun pair et ne crée aucun portefeuille. Le nom `status: verified` confirme seulement le chargement et la validation locale des données : il ne garantit pas que le nœud est au meilleur sommet mondial, qu'il a des pairs, que le réseau est honnête ou que les fonds sont sûrs.

## Contrats de sécurité pour les API suivantes (V1.2)

1. **Premier choix à faire :** IPC interne d'un conteneur desktop ou HTTP loopback (127.0.0.1 seulement). Dans le cas HTTP : contrôles Host/Origin et anti-DNS rebinding, token de session OS-éphémère, politique CORS refusant tout par défaut, rejet CSRF, délais et quotas ; jamais d'écoute 0.0.0.0.
2. Processus de signature Rust distinct du plan d'observabilité quand possible, confirmations UI et aucune possibilité de « signer automatiquement » depuis un endpoint d'exploration.
3. Version d'API `schema_version` explicite, séparation `testnet`/`regtest`, affichage erreur de protocole incompatible, champs optionnels et limites de pagination.
4. Réconciliation bloc/tx : consulter source nœud et statut canonique, invalider caches/index après réorganisation. Les identifiants de transaction et les montants restent des types précis (chaînes décimales ou entiers, pas nombres JS flottants).
5. Aucune connexion à un autre ordinateur sans accord visible de l'utilisateur. Les tests multi-nœuds sous CI ne constituent pas une démonstration d'infrastructure publique.
6. À chaque PR : tests malformés, volumes de réponses bornés, refus des données obsolètes, sécurité fichiers symlink/jonction, non-divulgation des logs, séparation réseau et état graphique.

## Première progression fonctionnelle

| Slice | Fonction concrète | Condition de fin |
| --- | --- | --- |
| V1.1 | Statut JSON local vérifié | Tests Python multi-OS + vrai nœud éphémère en CI + aucun port ni clé |
| V1.2 | Pont local read-only typé | Origin/session/limites adversariales ; pas de signer |
| V2.1 | Desktop dashboard et états réels | Vue hauteur/tip/état du nœud provenant de V1.2 ; maquettes clairement distinguées |
| V3.1 | Wallet GUI non custodial | Création/restauration avec clés chiffrées via Rust ; fonds jetables |
| V3.2 | Recevoir/envoyer signé | Tests de vrais paiements testnet, erreurs, reorgs, confirmations |
| V4.1 | Minage GUI contrôlé | Démarrer/pause/arrêter mineur réel et vérifier chaque bloc |
| V5.1 | Explorateur local | Recherche bloc/tx/adresse et historique via indexeur réconcilié |
| V6.1 | Gestion P2P | Pairs LAN opt-in, diagnostics, filtres et budgets |
| V20 | Intégration logicielle GitHub | E2E GUI → wallet → nœud P2P → mineur → explorateur |
| V21 | Deux PC réels + 1/4/24/72 h | Preuves humaines et ressources, à la **toute fin** |
| V22 | Revue E1 par une personne extérieure au projet | Ultime étape ; les revues du porteur et des agents restent des vérifications internes ; aucun feu vert fonds réels par anticipation |

**Décisions non prises :** Tauri vs autre conteneur, RPC vs IPC, distribution du portefeuille mobile, support Ledger, confidentialité et gouvernance futures. Les inscrire en ADR et valider les implications de sécurité avant implémentation ; ne pas modifier le consensus en passant.
