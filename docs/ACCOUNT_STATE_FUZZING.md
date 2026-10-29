# Fuzzing différentiel des transitions par comptes

La cible account_state produit jusqu'à huit opérations par entrée. Les transactions
Regtest sont signées avec une clé Ed25519 publique de test pour atteindre les
contrôles d'état, puis certaines autorisations sont corrompues. Les entrées font
varier financement, réseau, nonce, montant, frais, auto-transfert et restauration
d'un état initial. Chaque transition est comparée avec et sans cache de signature.

Assertions : résultats et états identiques, absence de mutation après rejet,
émission brute inchangée. La restauration conserve le cache de processus et
permet de tester le danger d'une décision d'état mémorisée.

La campagne dédiée dure 60 secondes, limite les entrées à 40 octets et conserve
journal, corpus et crashs en artefacts. Elle ne remplace pas une campagne longue
ni les tests ML-DSA-65 du testnet : ce générateur utilise le profil Ed25519
Regtest pour explorer rapidement les règles d'état communes. Il n'exerce pas
le choix de branche ou la persistance ; la restauration est une simulation.

Commande : cargo +nightly fuzz run account_state -- -max_total_time=60 -max_len=40 -print_final_stats=1
