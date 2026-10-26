# AURORA — état vérifié et chemin vers une version finale

**Branche de travail :** `feature/emission-schedule-v0`.  
**État :** testnet expérimental supervisé. La PR reste non fusionnée. Aucun fonds réels; aucune revendication de sécurité mainnet.

## 1. Éprouver le réseau et le stockage

### Déjà couvert par les contrôles automatisés

- Validation de transactions ML-DSA-65 v3, blocs, retarget borné, MTP, travail cumulé et réorganisations.
- Snapshot version 4 avec checksum, relecture et revalidation des branches; récupération depuis un snapshot précédent si le dernier est corrompu.
- Décodage hostile : tailles excessives, trames/inventaires malformés, handshake incompatible, bloc rejeté et limites de session.
- Scénario multi-processus : le smoke test mine et propage, inclut une transaction wallet, vérifie son statut et sa preuve Merkle, passe la hauteur 20 à travers deux fenêtres de retarget, redémarre un pair et synchronise les blocs manquants.
- Reprise après terminaison forcée entre des sauvegardes; essai d’échec d’écriture simulé par limite de taille de fichier sur Linux.
- Test reproductible déterministe d’entrées arbitraires pour les décodeurs de transactions, blocs, en-têtes, inventaires et listes de pairs.
- Cibles libFuzzer pour les décodeurs canoniques et les preuves Merkle; job GitHub Actions séparé.

### À compléter avant de prétendre à une robustesse élevée

- Campagnes fuzz longues et rapport de couverture; le job CI court ne remplace pas une campagne de plusieurs heures/jours.
- Plus de scénarios lents et malveillants : ralentissement volontaire, inventaire menteur, déconnexion au milieu d’une page, saturation simultanée et partitions.
- Essais de restauration depuis une sauvegarde hors machine, migrations de format réelles et test avec disque effectivement plein. La limite de taille du fichier reproduit une erreur d’écriture; elle ne simule pas une panne électrique ou un volume saturé.
- Réorganisations multi-nœuds concurrentes et synchronisation prolongée sur plusieurs processus.
- Vérification indépendante du retarget sous dérive d’horloge, longues pauses et rafales.

## 2. Revue indépendante — porte de sécurité non satisfaite

Le dossier [INDEPENDENT_REVIEW.md](INDEPENDENT_REVIEW.md) décrit le périmètre, les commandes et les limites connues. Aucune revue indépendante n’a été réalisée. Les contrôles CI, ce travail d’implémentation et les outils automatisés ne comptent pas comme une telle revue.

Il faut un auditeur humain indépendant, un commit épinglé, un rapport écrit avec constats classés, puis des corrections suivies de nouveaux tests. Le dépôt est privé et aucun reviewer indépendant n’est actuellement désigné; aucune demande de revue n’a été envoyée à une personne non identifiée.

## 3. Décisions de protocole et économie

Les paramètres économiques demandés par le porteur sont codés : plafond brut de 500 millions AUR, aucune préémission, émission proportionnelle décroissante, 5 % crédités à l’adresse de trésorerie de test, 95 % au mineur, frais partagés 50/50 et burn non réutilisable sous le plafond.

Deux sujets doivent rester explicitement séparés de ces règles :

- L’adresse de trésorerie reçoit la part prévue, mais il n’existe ni contrat, ni gouvernance, ni règles de dépense/audit. Ne pas présenter cette adresse comme une trésorerie autonome opérationnelle avant qu’une politique de contrôle et d’utilisation soit décidée puis revue.
- Le protocole actuel ne saisit et ne recycle pas les soldes inactifs. Toute récupération d’actifs change les droits de propriété; elle exige une décision explicite sur le seuil, les preuves, le recours et les protections contre le vol de clés.
- DID, gardiens et récupération restent au stade de conception hors consensus dans [DID_RECOVERY_DESIGN.md](DID_RECOVERY_DESIGN.md). Ils ne peuvent être ajoutés à l’état des comptes qu’après conception détaillée, revue et décision du porteur.

## 4. Stabiliser le testnet

La configuration locale reproductible et son guide sont disponibles dans [TESTNET_GUIDE.md](TESTNET_GUIDE.md). Les paramètres de preuve de travail sont délibérément faibles et la topologie demande des pairs connus. La CI valide un scénario local, pas un réseau public exploité dans la durée.

Avant de déclarer un testnet public stable, il faut choisir et héberger des nœuds semences, réaliser une campagne multi-hôtes prolongée, surveiller la disponibilité/convergence, geler une version protocolaire, et documenter les mises à jour, sauvegardes, restauration et signalement d’incident. Aucun hôte public ni opérateur externe n’est fourni dans ce dépôt.

## 5. Applications — après les portes précédentes

Ces éléments sont volontairement hors de la phase actuelle. Ils commencent après les essais prolongés, la revue indépendante, les corrections et le gel du protocole :

1. **Portefeuille quotidien** : chiffrement au repos, sauvegarde/restauration vérifiable, gestion des secrets, puis tests d’ergonomie. Le wallet actuel est un outil testnet dont le fichier de clé est en clair.
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