# ADR V2-001 — Tauri v2 / React pour ARETOVUS Desktop

Statut : **choix provisoire pour un prototype expérimental**, à revalider après les tests multi-OS et l'analyse de sécurité. Référence #181, PR #194 et feuille #178. Aucun nouveau protocole ni changement de consensus.

## Contexte

Le noyau ARETOVUS est en Rust ; le navigateur ne sait pas parler directement au protocole TCP P2P et ne doit ni manipuler les clés privées ni lancer le minage. Nous devons afficher des états validés issus du nœud sans créer une API HTTP publique involontaire.

## Choix et alternatives

- **Choix prototype : React/TypeScript + Vite** pour l'interface ; **Tauri v2** pour héberger le code Rust et utiliser des commandes IPC locales. L'interface appelle un contrat de lecture seule versionné ; aucune extension web ni script distant.
- Alternative Electron : écosystème mature mais moteur de navigateur embarqué, surface/mémoire accrues ; à reconsidérer selon le support WebView réel.
- Alternative GUI Rust native : front sans JavaScript, mais changement d'outillage et de capacités UI. Peut redevenir préférable pour un wallet à fortes exigences.
- Aucun arbitrage ne permet de stocker seeds/privkeys dans JavaScript ou de désactiver les vérifications de données pour améliorer la fluidité.

## Frontières de confiance

1. Le nœud ARETOVUS et ses règles de consensus restent seuls responsables de valider les blocs/transactions.
2. Le premier pont binaire Rust (#192) charge un snapshot existant et valide son identité native. Contrat V1 strict et read-only. Ce pont **ne prouve pas** la synchronisation globale ni le caractère définitif d'une transaction.
3. Le conteneur Tauri démarre un binaire compagnon fixe situé à côté de son exécutable; il n'accepte ni commande shell ni binaire choisi dans PATH. Il termine les processus trop lents; l'intégrité du répertoire d'installation et le packaging du sidecar restent des risques à examiner.
4. Le frontend React ne reçoit que des champs publics validés et ne montre jamais de wallet/solde inventé. Les erreurs ne contiennent ni chemin personnel, ni stderr brut. Aucune clé privée ne traverse IPC dans cette tranche.
5. L'application actuelle n'a **ni mode minage, ni API socket, ni RPC de signature, ni connexion aux pairs depuis JS**. Tout futur endpoint d'écriture exigera une politique d'autorisation indépendante et une action explicite de l'utilisateur.

## Points bloquants avant une distribution

- Verrouillage des dépendances npm et Rust du conteneur, inventaire SBOM et revue de provenance.
- Construire, tester et empaqueter le binaire compagnon sur Windows/Linux/macOS sans charger de code arbitraire depuis PATH.
- Tester l'exécution Tauri réelle sur les trois OS, chemins non ASCII/avec espaces, UI suspendue, binaire manquant/modifié, crash et timeout, absence de secrets dans logs.
- Évaluer CSP et surface WebView, compatibilité WebView2/GTK/WebKit/macOS ; mesurer CPU/RAM en polling.
- Signer les exécutables et archives lorsqu'une version destinée à des utilisateurs sera envisagée.
- Réexamen humain du canal IPC avant d'y connecter le wallet et le minage.

## Suites

Petites PR indépendantes : fiabiliser sidecar et contrats, tester Tauri natif, ajouter vues synchronisation/pairs depuis données vérifiables, puis portefeuille graphique via Rust sans exposer de clés. Les essais physiques à deux ordinateurs et l'audit externe indépendant restent aux dernières phases demandées par le responsable. **Pas de fonds réels.**
