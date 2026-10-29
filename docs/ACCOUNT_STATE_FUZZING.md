# Fuzzing différentiel des transitions par comptes

La cible account_state produit jusqu'à huit opérations par entrée. Les transactions
Regtest sont signées avec une clé Ed25519 publique de test pour atteindre les
contrôles d'état, puis certaines autorisations sont corrompues. Les entrées font
varier financement, réseau, nonce, montant, frais, auto-transfert et restauration
d'un état initial. Chaque transition est comparée avec et sans cache de signature.

Assertions : résultats et états identiques, absence de mutation après rejet,
émission brute inchangée. La restauration conserve le cache de processus et
permet de tester le danger d'une décision d'état mémorisée.

La campagne dédiée dure 300 secondes, limite les entrées à 40 octets et conserve
journal, corpus et crashs en artefacts. Elle ne remplace pas une campagne longue
ni les tests ML-DSA-65 du testnet : ce générateur utilise le profil Ed25519
Regtest pour explorer rapidement les règles d'état communes. Il n'exerce pas
le choix de branche ou la persistance ; la restauration est une simulation.

Commande : cargo +nightly fuzz run account_state -- -max_total_time=300 -max_len=40 -print_final_stats=1

## Critères de livraison

Huit entrées de référence sont versionnées dans fuzz/seeds/account_state :
séquence valide, signature altérée, rejeu de nonce, restauration avec cache chaud,
solde insuffisant, mauvais réseau, auto-transfert et montant nul. Elles sont
rejouées individuellement avant chaque campagne, puis injectées dans le corpus.

La cible vérifie aussi apply_batch : un préfixe valide suivi d'une transaction
invalide doit laisser le registre initial inchangé ; un lot valide doit produire
le même état que les transitions individuelles. Les frais retournés ne sont
pas comparés par cet invariant, mais les états et l'émission le sont.

Le workflow restaure le corpus des campagnes précédentes pour la même version
de cible, le complète pendant cinq minutes, puis rejoue l'ensemble enrichi
sans mutation. Corpus, logs, commit et version du compilateur sont conservés
en artefacts ; les graines de référence restent dans Git. Un crash fait échouer
le job et doit devenir une régression versionnée après correction.

Cette livraison couvre la cible, les invariants, le corpus initial, la reprise
et le rejeu automatiques. La durée de cinq minutes reste une campagne bornée ;
elle ne prouve pas l'absence de défauts.
