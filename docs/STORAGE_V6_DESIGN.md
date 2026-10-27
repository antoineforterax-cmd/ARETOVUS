# AURORA — proposition de stockage et de synchronisation v6

**Statut : implémentation partielle; CI v6 verte sur `21a24c7`.** La branche écrit déjà un manifeste et des pages v6, lit v4/v5, et sert les blocs dans un ordre d’arrivée stable. Le P2P utilise maintenant un curseur u64 et rouvre les sessions avant leurs quotas. Les états complets ont été réduits à des points de contrôle tous les 256 blocs et au sommet canonique, mais leur croissance et la copie de chaîne par page restent à mesurer. Les jobs GitHub Actions ont repris et la [CI v6](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40485261614) a réussi; l’essai de volume manque toujours. Aucun fonds réel.

## Problème observé

Le stockage v5 écrit un index monolithique contenant tous les identifiants de blocs, borné à 64 MiB et à 100 000 blocs. Il recharge tous les blocs et garde un état complet par bloc en mémoire. L’ancien protocole P2P annonçait un total u32, un curseur u32 et plafonnait l'inventaire à 100 000; une session est limitée à 256 MiB et 15 minutes. L’ancien ordre réseau était trié par hauteur puis hash : l’arrivée ultérieure d’une branche ancienne insérait des blocs *avant* un curseur déjà téléchargé. Le service de pages utilise désormais l’ordre d’arrivée; une restauration v6 le conserve. Augmenter les constantes sans modifier ces propriétés laisserait un rattrapage incorrect ou excessivement coûteux.

## Règles visées

1. **Aucune limite de durée de vie à 100 000 blocs.** Les limites de trame, de page, de temps et de mémoire restent explicites. Un nœud peut manquer de ressources, mais un bloc valide ne doit pas être rejeté simplement parce que le total historique franchit une constante.
2. **Ordre d'inventaire append-only.** Chaque bloc accepté reçoit un numéro u64 croissant; son parent a toujours un numéro inférieur. Le numéro et le hash sont persistés. L'ordre est local au nœud et n'entre pas dans le consensus. Un bloc de branche secondaire arrivé tard est ajouté en fin d'inventaire, sans déplacer les anciens curseurs.
3. **Index v6 paginé.** Les pages immuables actuelles contiennent jusqu’à 4 096 hashes de blocs avec décalage u64 et checksum. Le manifeste engage le compte u64, les paramètres et les hashes des pages; le sommet canonique est recalculé par validation au chargement. Il reste à vérifier l’atomicité sur pannes et à mesurer les performances. Les fichiers de blocs restent adressés par hash. Trois générations de manifeste sont conservées pour la reprise. Une génération ne peut référencer qu'une page intégralement synchronisée sur disque.
4. **Reprise P2P par curseur u64.** Le pair annonce une génération d'inventaire et un compte; le client demande des pages bornées à partir d'un numéro. Il peut fermer puis rouvrir la connexion après épuisement d'un budget de session et poursuivre avec ce numéro. Le serveur vérifie la continuité du curseur; si la génération a changé, le client vérifie le dernier couple numéro/hash déjà reçu puis reprend ou recommence de façon sûre. Les pages sont appliquées atomiquement après validation consensus et sauvegarde.
5. **États bornés.** Garder les blocs et métadonnées nécessaires aux réorganisations, mais remplacer le snapshot complet de l'état pour *chaque* bloc par des points de contrôle et une reconstruction vérifiée depuis un ancêtre. Les états des sommets récents peuvent être mis en cache avec un budget mesuré. Aucun horizon de réorganisation n'est ajouté silencieusement.
6. **Compatibilité explicite.** Lire v5 et écrire v6 après migration réussie; identifier le nouveau protocole P2P par une nouvelle empreinte. Une copie du dossier v5 est requise avant migration. Les soldes et la genèse ne changent pas; le format de stockage et l'inventaire réseau changent.

## Séquence d'implémentation

- Fait dans le code : ordre d’acceptation persistant et test d’une branche ancienne qui ne décale pas le curseur.
- Fait et vérifié par CI : écriture/lecture du manifeste et des pages v6, test de corruption et migration v5. Restent l’essai de coupure au milieu d’une page, la restauration après plusieurs générations et la mesure de volume.
- API P2P u64, reconnexion de session avec contrôle du dernier hash et tests de pair incohérent écrits; leur CI doit encore passer. Reste une chaîne au-delà de 100 000 blocs.
- Des points de contrôle tous les 256 blocs et un rejeu borné ont été ajoutés avec un test de branche profonde et récompenses; la CI et les mesures mémoire restent à faire. La copie de chaîne par page de sync reste coûteuse.
- Mesurer sur une chaîne synthétique de plus de 100 000 blocs, incluant branches secondaires, redémarrage, restauration et sync depuis zéro; conserver CPU, RAM, disque, durée et hashes des sommets dans le rapport d'essai.

**Critère de sortie :** une même révision passe CI, fuzzing, essai de volume et reprise multi-nœuds avec une chaîne dépassant 100 000 blocs connus. Ce document seul ne satisfait pas ce critère.
