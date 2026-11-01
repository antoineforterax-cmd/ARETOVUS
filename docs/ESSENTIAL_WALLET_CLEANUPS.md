# Petites corrections essentielles du wallet — 29 octobre 2026

Ce lot corrige des lacunes du CLI expérimental existant. Il s'appuie sur
les objectifs de validation, de limites de ressources et de protection du
portefeuille du cahier des charges (§2, §5, §7 et §8).

| Suivi | Problème | Correction et preuve attendue |
| --- | --- | --- |
| #30 | Anciennes commandes : options ignorées, doublons ambigus | Validation stricte de toutes les commandes avant leur exécution ; tests CLI pour options inconnues, répétées ou sans valeur, et maintien de l'aide valide |
| #31 | Le client RPC pouvait accepter un reçu portant un autre txid | Contrôle centralisé contre les octets de la transaction préparée ; faux serveur TCP avec reçu correct, mensonger, tronqué ou trop long |
| #32 | Décodage URI trop permissif en ressources et caractères d'affichage | URI limitée à 1 024 octets, mémo encodé à 384 octets avant allocation, contenu UTF-8 à 128 octets ; refus des contrôles directionnels, maintien des lettres Unicode visibles |
| #33 | Addition directe à Instant susceptible de paniquer | checked_add avec erreur explicite ; tests zéro et u64::MAX, dont une invocation CLI qui doit échouer avant réseau |
| #34 | Copies temporaires de mots de passe, graines et clés dérivées | Tampons Zeroizing et effacement des copies locales de graines ; tests existants de chiffrement, sauvegarde, secours, identité et signature à repasser |

Les résultats effectifs, le SHA et les limites de validation sont consignés
dans la PR du lot. Chaque changement fonctionnel conserve son propre commit.

## Portée et compatibilité

Aucune règle de consensus, valeur monétaire, structure de transaction,
format ARETOVK3/ARETOVK4, graine de récupération ou trame réseau ne change.
Aucune dépendance supplémentaire ni télémétrie n'est ajoutée.

Les commandes CLI qui utilisaient une faute de frappe, un argument non
reconnu ou deux fois la même option échouent désormais. Les appels valides
gardent leur comportement. Les nouveaux refus d'URI concernent des entrées
trop grandes ou contenant des contrôles de direction ; cela ne change pas
les transactions de consensus ni les mémos UTF-8 visibles légitimes.

La vérification du reçu n'est pas une preuve de confirmation : un nœud peut
avoir reçu le paiement puis renvoyer un faux reçu, ou la connexion peut
échouer après soumission. Consulter le statut du txid préparé avant de
réémettre ou de signer un autre paiement. Le contrôle du txid ne prouve pas
l'honnêteté d'un nœud dont la réponse correspond au format attendu.

Le correctif de wait évite l'overflow du calcul de délai. Les appels RPC
conservent leurs propres délais réseau : l'argument de polling n'est pas
un budget global strict pour la durée totale de la commande.

L'effacement vise les tampons locaux identifiés. Il ne prouve pas
l'effacement de toutes les copies internes des bibliothèques, du système,
des snapshots, du swap ou du disque. Une revue indépendante reste requise.

## Suivi global

Le [tableau de finalisation](FINALIZATION_CHECKLIST.md) et la
[roadmap](planning/ROADMAP.md) différencient les fonctions sur PR de celles
fusionnées. Ce lot ne remplace pas la correction des risques P0 réseau,
les essais de grande échelle, une campagne de 72 h, les essais physiques
ni un audit humain. Les nouvelles issues restent ouvertes jusqu'à
l'intégration effectivement décidée ; une CI verte n'est pas une fusion.

Le retour à un ancien wallet réintroduit les tolérances et lacunes décrites
ici, sans migration de données nécessaire. Le testnet demeure expérimental.
