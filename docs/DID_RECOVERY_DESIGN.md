# Conception DID, authentification et récupération — AURORA

**Statut :** proposition pour le testnet expérimental, pas une règle de consensus active.  
**Portée :** définir les comportements avant toute extension de l’état des comptes. Le format actuel v3 reste inchangé jusqu’à l’implémentation et à la revue du nouveau profil.

## Principes

1. Un DID est une déclaration d’identité facultative; il ne donne aucun droit de dépense et n’est jamais nécessaire pour créer ou utiliser un compte.
2. L’identité publique et les attributs DID restent hors chaîne. Le consensus ne stocke pas de nom, courriel, document d’identité, URL de profil ou preuve personnelle.
3. Le contrôle des fonds est toujours établi par une autorisation cryptographique vérifiée par consensus. Une preuve challenge-response n’est pas une transaction et ne change aucun solde.
4. Les gardiens ne peuvent ni voir ni déplacer les fonds en dehors d’une récupération explicitement autorisée par le seuil enregistré.
5. La récupération d’un compte inactif ne saisit pas les fonds. Aucune règle d’inactivité ou de recyclage ne fait partie de cette proposition.

## Défi-réponse DID

Le vérificateur génère un nonce aléatoire de 32 octets à usage unique. Le portefeuille signe l’encodage canonique de :

- version du message AURORA-DID-AUTH-v1;
- identifiant de réseau;
- compte AURORA;
- DID optionnel présenté (UTF-8 normalisé NFC, maximum 512 octets);
- nonce;
- audience exacte (origine ou service qui vérifie, maximum 256 octets);
- action/purpose (maximum 64 octets);
- expiration Unix en secondes.

Le digest utilise un domaine de hachage distinct des transactions et des demandes de récupération. Le vérificateur exige une signature ML-DSA-65 valide, une expiration au plus 5 minutes dans le futur, une audience correspondant exactement à sa propre identité, un nonce émis par lui et non encore consommé. Le nonce est marqué utilisé même si le vérificateur rejette le DID ou l’action. Une réponse valide prouve seulement que le répondant contrôlait la clé autorisée au moment du défi; elle ne certifie pas la vérité d’un DID ni l’identité civile.

Les vérificateurs ne doivent pas accepter un texte signé que l’utilisateur fournit sans nonce frais. Aucun message challenge-response ne peut être rejoué comme transaction car les domaines et encodages sont distincts.

## Clés de dépense stables et récupération

Une récupération exige une identité de compte stable distincte de la clé de dépense active. Une adresse actuelle dérivée directement de la clé publique ne peut pas être réattribuée à une nouvelle clé sans changer le compte; le nouveau profil devra donc séparer l’identifiant stable du compte et l’empreinte de la clé autorisée. Cette transition est incompatible et requiert un nouveau profil de transaction/état et une activation de consensus explicite.

L’état proposé par compte comporte uniquement :

- l’empreinte de la clé de dépense active;
- une politique optionnelle avec 2 à 5 comptes gardiens distincts, un seuil compris entre 2 et le nombre de gardiens, et un compteur de version;
- au plus une demande de récupération en attente (empreinte de la nouvelle clé, nonce de politique, hauteur d’activation, hauteur d’expiration et approbations).

Les clés publiques complètes restent dans les transactions autorisées, pas dans un tableau de chaînes d’identité.

### Configuration et modification des gardiens

- Le titulaire de la clé active configure ou modifie la politique par une transaction v4 signée.
- La liste ne peut contenir le compte contrôlé lui-même, des doublons, ni des comptes sans clé de dépense établie.
- Une modification incrémente la version de politique et invalide toutes les demandes en attente.
- La transaction de configuration ne s’applique qu’après la vérification complète de l’autorisation et des limites ci-dessus.
- La suppression de la politique exige la clé active; la récupération ne peut pas supprimer ou remplacer la liste de gardiens elle-même.

### Démarrage et exécution d’une récupération

1. Une demande doit être signée par le seuil de gardiens sur un message qui engage le compte cible, la nouvelle clé publique/empreinte, le réseau, la version de politique, le nonce du compte, la hauteur d’expiration et le seuil.
2. Les approbations doivent provenir de gardiens distincts, être valides pour la même demande et ne sont pas réutilisables.
3. La demande est inscrite au consensus et devient exécutable après 1 440 blocs (réglage de prototype testnet). Elle expire 10 080 blocs après son inscription si elle n’est pas exécutée.
4. Pendant le délai, l’ancienne clé peut annuler la demande par une transaction normale et continuer à utiliser les fonds. Une annulation invalide la demande immédiatement.
5. À la hauteur d’activation, toute transition consomme le nonce suivant et remplace uniquement la clé active. Le solde, l’identité stable du compte, le nonce et la politique de gardiens restent identiques.
6. La nouvelle clé doit signer une preuve d’acceptation de récupération attachée à la transaction d’activation.
7. Une réorganisation rejoue l’inscription, l’annulation et l’activation selon la chaîne gagnante; aucun état de récupération ne vient d’une branche abandonnée.

Les nombres 1 440/10 080 sont des paramètres de testnet, pas des délais pour mainnet. Un futur réseau doit définir ses délais en unités de consensus et analyser le temps réel attendu, les réorganisations, l’inactivité et les scénarios de censure.

## Menaces et règles à régler avant activation

Avant tout code de consensus, une revue indépendante devra examiner : compromission simultanée du seuil, collusion ou coercition de gardiens, remplacement d’adresse, course avec des transactions normales, nonces parallèles, réorganisation pendant le délai, blocage/censure, renouvellement avant expiration, rotation des clés post-quantiques, perte des clés, divulgation DID, limites d’encodage et déni de service.

Le compte de trésorerie et les éventuels comptes protocolaires ne bénéficient d’aucune exception aux règles. Aucun mécanisme anti-sinking ne peut dépenser, geler ou réattribuer un compte inactif sans décision distincte du porteur, spécification des preuves/recours et revue indépendante.

## Non-engagement

Ce document n’autorise pas l’ajout de champs à AccountState, ne définit pas un standard DID externe complet et ne constitue ni identité vérifiée ni récupération utilisable. Toute activation de consensus restera bloquée jusqu’à la revue cryptographique/protocolaire indépendante et à l’acceptation explicite du porteur du projet.
