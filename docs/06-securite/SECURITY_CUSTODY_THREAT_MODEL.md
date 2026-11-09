# ARETOVUS — menace de détournement des signatures et de la garde

**31 octobre 2026 — testnet expérimental.** Ce document n'est pas un audit ni une garantie de sécurité. Pas de vrais fonds.

## Événements distincts, erreurs à ne pas confondre

- **BNB Chain, octobre 2022 :** forge de preuves sur BSC Token Hub (pont inter-chaînes) et création frauduleuse de BNB. Source : https://www.bnbchain.org/en/blog/bnb-chain-ecosystem-update. **ARETOVUS ne doit pas activer de pont** sans spécification des preuves, vérification indépendante et audit.
- **Bybit, février 2025 :** détournement de l'interface de signature multi-signature Safe, avec validations humaines d'une autre opération que celle effectivement signée. Source FBI : https://www.fbi.gov/investigate/cyber/alerts/2025/north-korea-responsible-for-1-5-billion-bybit-hack. **Plusieurs signatures ne suffisent pas** si les signataires se fient au même écran compromis.
- **Binance, octobre 2026 :** compromission des systèmes de retraits / autorisations de hot wallets, pertes estimées à 6,3 Md USD. Investigation en cours; ne pas attribuer avec certitude le mécanisme final ou la responsabilité. Source officielle : https://www.binance.com/en/support/announcement/detail/d3306d9dca5243548c813b605e1d1751.

## Barrières disponibles et ajoutées dans cette branche

1. Les transactions v3 Testnet sont signées en ML-DSA-65 avec domaine réseau; l'identité du destinataire est représentée par son identifiant canonique complet.
2. **Nouveau contrôle d'intention** pour les commandes interactives `send`, `sign-offline` et `broadcast` : l'opérateur doit confirmer explicitement réseau, destinataire complet, montant, frais et nonce. Un refus, EOF, entrée partielle, caractère supplémentaire ou altération d'un champ refuse l'autorisation.
3. **Vérification après signature** avant sauvegarde hors ligne ou émission RPC : le contenu signé doit toujours correspondre à l'intention, et la signature est revérifiée. Les tests tentent des substitutions y compris avec un payload resigné et une altération cryptographique.
4. La commande `--dry-run` conserve un aperçu sans signer. Les limites facultatives `.limits` empêchent certains montants et frais excessifs lorsqu'elles sont effectivement présentes et intègres. Les signatures hors ligne n'exigent pas que la clé privée passe sur le PC connecté.
5. `broadcast` vérifie l'authenticité du fichier signé, le nonce et le solde côté nœud, et la réponse RPC doit identifier l'exacte transaction préparée.

## Limites de sécurité — bloquants avant toute utilisation réelle

- **Le drapeau `--yes` ignore toute saisie humaine** pour compatibilité des tests et scripts. **Ne l'utilisez pas pour autoriser des transferts sensibles.** Une application hostile peut invoquer des commandes signantes sans interaction si elle a accès à la clé déchiffrée ou à un terminal automatisé.
- **Un terminal, un logiciel de wallet ou un OS compromis peut mentir à l'écran et modifier les entrées.** Le texte de confirmation reste sur le même appareil; ce n'est **pas** un affichage indépendant de matériel sécurisé. Une vérification de destination/montant/nonce sur un appareil indépendant est à construire et à auditer.
- Une clé volée peut signer des transactions acceptées par le réseau. Le consensus ne peut pas deviner qu'une signature pourtant valide a été provoquée par une intrusion. Le wallet actuel **n'offre pas** de HSM, multi-signature distribuée, limites de débit de retraits imposées par consensus, liste blanche consensuelle, mécanisme de gel, révocation ou coffre social.
- Les limites locales en fichier `.limits` peuvent être retirées/altérées par un système compromis; **ce ne sont pas des restrictions de consensus ni une défense contre un vol de clé.**
- Des clés et soldes détenus par un échange centralisé impliquent un risque de garde indépendant du consensus ARETOVUS. Ni l'émission, ni la blockchain, ni un test unitaire ne peuvent garantir contre un détournement de son service de retraits.
- Un futur pont entre chaînes multiplierait les frontières de confiance, comme l'attaque BNB Chain 2022; **aucun pont n'est autorisé** dans le périmètre actuel.

## Exigences pour une garde à risque réel (non implémentées)

- Signataire hors ligne sur matériel indépendant affichant **les vrais octets décodés** de destinataire, montant, frais, réseau, nonce et type d'opération; contrôle croisé du hash/txid et du code signé.
- Séparation des rôles, clefs froides et chaudes, rotation, privilège minimal, reproductibilité du build, monitoring des opérations de signature et réponse aux incidents, protections contre détournement de mise à jour et supply chain.
- Politique de multisignature/quotas et transaction simulée à **spécifier sans modifier le consensus courant**, puis implémenter/tester sur nouveau profil compatible. Pas d'hypothèse selon laquelle 2FA ou un nombre fixe de signataires élimine la tromperie.
- Tests adversariaux : UI trompeuse, modifications de transaction après revue, fausse réponse RPC, compromis du poste opérateur, fichier signé copié, fuite de clé, altération du code, redémarrage, réorganisations et défaillances de sauvegarde.
- Audit indépendant qualifié du wallet, des algorithmes de signature, du protocole, du réseau et de la chaîne d'approvisionnement avant toute déclaration mainnet.

**Invariant de politique :** aucune mesure locale ne doit être présentée comme capable de bloquer un attaquant en possession d'une clé valide. Jamais de véritables fonds sur ce testnet.
