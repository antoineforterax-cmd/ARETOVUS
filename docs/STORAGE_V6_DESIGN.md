# AURORA — proposition de stockage et de synchronisation v6

**Statut : proposition de travail, non activée.** Le testnet v3 utilise toujours le stockage v5 et refuse plus de 100 000 blocs connus. Ce document fixe l'architecture à implémenter et à tester avant de lever cette borne. Aucun fonds réel.

## Problème observé

Le stockage v5 écrit un index monolithique contenant tous les identifiants de blocs, borné à 64 MiB et à 100 000 blocs. Il recharge tous les blocs et garde un état complet par bloc en mémoire. Le protocole P2P annonce un total u32, un curseur u32 et plafonne l'inventaire à 100 000; une session est limitée à 256 MiB et 15 minutes. L'ordre actuel est trié par hauteur puis hash : l'arrivée ultérieure d'une branche ancienne insère des blocs *avant* un curseur déjà téléchargé. Augmenter les constantes sans modifier ces propriétés laisserait un rattrapage incorrect ou excessivement coûteux.

## Règles visées

1. **Aucune limite de durée de vie à 100 000 blocs.** Les limites de trame, de page, de temps et de mémoire restent explicites. Un nœud peut manquer de ressources, mais un bloc valide ne doit pas être rejeté simplement parce que le total historique franchit une constante.
2. **Ordre d'inventaire append-only.** Chaque bloc accepté reçoit un numéro u64 croissant; son parent a toujours un numéro inférieur. Le numéro et le hash sont persistés. L'ordre est local au nœud et n'entre pas dans le consensus. Un bloc de branche secondaire arrivé tard est ajouté en fin d'inventaire, sans déplacer les anciens curseurs.
3. **Index v6 paginé.** Des pages immuables de taille bornée contiennent les couples numéro/hash et un checksum; un manifeste atomique engage l'ordre, le compte u64, la genèse, les paramètres, le sommet canonique et les hashes des pages. Les fichiers de blocs restent adressés par hash. Trois générations de manifeste sont conservées pour la reprise. Une génération ne peut référencer qu'une page intégralement synchronisée sur disque.
4. **Reprise P2P par curseur u64.** Le pair annonce une génération d'inventaire et un compte; le client demande des pages bornées à partir d'un numéro. Il peut fermer puis rouvrir la connexion après épuisement d'un budget de session et poursuivre avec ce numéro. Le serveur vérifie la continuité du curseur; si la génération a changé, le client vérifie le dernier couple numéro/hash déjà reçu puis reprend ou recommence de façon sûre. Les pages sont appliquées atomiquement après validation consensus et sauvegarde.
5. **États bornés.** Garder les blocs et métadonnées nécessaires aux réorganisations, mais remplacer le snapshot complet de l'état pour *chaque* bloc par des points de contrôle et une reconstruction vérifiée depuis un ancêtre. Les états des sommets récents peuvent être mis en cache avec un budget mesuré. Aucun horizon de réorganisation n'est ajouté silencieusement.
6. **Compatibilité explicite.** Lire v5 et écrire v6 après migration réussie; identifier le nouveau protocole P2P par une nouvelle empreinte. Une copie du dossier v5 est requise avant migration. Les soldes et la genèse ne changent pas; le format de stockage et l'inventaire réseau changent.

## Séquence d'implémentation

- Ajouter l'ordre d'acceptation persistant et tester qu'une branche ancienne n'invalide pas un curseur existant.
- Écrire/lire le manifeste et les pages v6, vérifier checksums, atomicité, coupure au milieu d'une page, restauration des trois générations et migration v5.
- Introduire l'API P2P u64 et la reconnexion de session, puis tester une chaîne au-delà de la petite borne injectée dans les tests.
- Réduire la mémoire des états sans modifier la validation des blocs, les récompenses ni la sélection au travail cumulé.
- Mesurer sur une chaîne synthétique de plus de 100 000 blocs, incluant branches secondaires, redémarrage, restauration et sync depuis zéro; conserver CPU, RAM, disque, durée et hashes des sommets dans le rapport d'essai.

**Critère de sortie :** une même révision passe CI, fuzzing, essai de volume et reprise multi-nœuds avec une chaîne dépassant 100 000 blocs connus. Ce document seul ne satisfait pas ce critère.
