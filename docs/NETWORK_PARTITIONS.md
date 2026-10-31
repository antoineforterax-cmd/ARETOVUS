# Partitions réseau répétées

Chantier P1 « pairs hostiles et partitions » de FINALIZATION_CHECKLIST.md.
Le lanceur crée deux serveurs sans pair configuré et fait miner deux blocs côté A,
un bloc concurrent côté B. Les récompenses sont envoyées à des comptes différents
pour éviter de produire accidentellement les mêmes blocs.

Les serveurs sont redémarrés avec un pair réciproque. Aucune commande sync ne
force leur convergence : les threads de synchronisation de fond doivent rejoindre
le même sommet. Le test exige hauteur, hash et travail identiques, ainsi que
la conservation de toutes les branches secondaires. Un pair neuf télécharge
ensuite l'historique et doit obtenir exactement le même état vérifié.

Trois cycles successifs réutilisent les dossiers des nœuds et mineurs. À chaque
cycle, le résultat sur disque est revérifié après arrêt. Les hauteurs canoniques
attendues sont 2, 4 et 6 ; le nombre total de blocs connus, genèse comprise,
est 4, 7 puis 10. Les mineurs restent donc sous le premier retarget pour isoler
la convergence et la réorganisation des effets de difficulté.

Commande :
python3 scripts/testnet_partitions.py --binary target/debug/aretovus-node --output target/partitions --rounds 3

Le dossier de sortie doit être neuf : aucun résultat précédent n'est effacé.
Les journaux et results.json sont conservés par le workflow network-partitions.
Le délai de convergence maximal est 45 secondes par cycle. Tous les sous-processus
serveurs sont arrêtés dans le bloc finally, même après échec.

La partition est réalisée en configurant des groupes de pairs disjoints, puis en
redémarrant les serveurs avec de nouveaux pairs. Elle n'utilise pas un pare-feu
et ne reproduit pas les pertes de paquets d'une liaison Internet. Cet essai
multi-processus sur un hôte ne remplace ni un essai multi-machines ni 72 heures
d'endurance ; il n'ajoute pas de paiements signés pendant les coupures.
