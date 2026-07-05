# Quark (QRK) — Whitepaper

## 1. Introduction

Quark (QRK) est une cryptomonnaie expérimentale basée sur un mécanisme de Proof-of-Stake original : le **Hadron Staking**. Le protocole s'inspire de la physique des particules pour lier la sécurité du réseau à un système de collection et de combinaison d'actifs numériques (les Quarks et Hadrons).

## 2. Motivation

Les systèmes PoS classiques lient le pouvoir de validation à la simple détention de jetons. Quark introduit une couche de gamification : les validateurs doivent assembler des combinaisons spécifiques de Quarks (Up, Down, Charm, Strange, Top, Bottom) pour forger des Hadrons, qui déterminent leur poids de sélection.

## 3. Architecture

### 3.1 Blockchain

- Blocs liés par hash SHA-256.
- Chaque bloc contient : index, timestamp, transactions, hash précédent, hash, validateur.
- Validation de chaîne par vérification récursive des hash.

### 3.2 Comptes et transactions

- Clés ed25519 (paire publique/privée).
- Adresses encodées en Base58.
- Transactions signées et vérifiées cryptographiquement.
- Modèle de compte (balance-based), pas UTXO.
- Frais de transaction définis par l'émetteur.

### 3.3 Quarks et Hadrons

- 6 types de Quark : Up, Down, Charm, Strange, Top, Bottom.
- Chaque bloc forgé distribue 3 Quarks aléatoires au validateur sélectionné.
- Combinaisons valides :
  - Up + Up + Down → **Proton**
  - Up + Down + Down → **Neutron**
- Autres combinaisons : invalides, quarks perdus.

### 3.4 Consensus (Proof-of-Stake)

- Sélection pondérée des validateurs, via un tirage combinant le hash du bloc précédent et une seed propre à chaque validateur (mécanisme simplifié type commit-reveal).
- Poids = somme des valeurs des Hadrons stakés (Proton = 3, Neutron = 2, plafonné à 30 par validateur) + 1 (poids de base).
- Résolution de forks : la chaîne valide la plus longue l'emporte (vérification de validité avant adoption).

### 3.5 Émission monétaire

- Récompense de bloc initiale : 1000 QRK.
- Halving tous les 10 blocs (récompense divisée par 2).

### 3.6 Réseau

- Communication TCP pair-à-pair basique (port par défaut 8080).
- Diffusion (broadcast) de la chaîne après chaque bloc forgé aux pairs connus.
- Synchronisation manuelle ou automatique avec vérification de validité avant adoption d'une chaîne reçue.
- Découverte de pairs basique via l'API (`GET /peers`) au démarrage d'un nœud.

### 3.7 API

- API HTTP (port 3000) exposant :
  - `GET /chain` — état complet de la blockchain (JSON)
  - `GET /peers` — liste des pairs connus (JSON)
  - `GET /` — explorateur de blocs basique (HTML)

### 3.8 Sécurité des wallets

- Clé privée chiffrée avec AES-256-GCM.
- Dérivation de clé par mot de passe via Argon2.
- Stockage local (fichier `.key`) par adresse.

### 3.9 Tests

- Tests unitaires sur chaque module (bloc, transaction, ledger, wallet, validateur, quark/hadron).
- Tests property-based (proptest) sur la vérification des transactions.

## 4. Limites connues

- Découverte de pairs basique (un seul point d'entrée, pas de gossip multi-hop).
- Résistance Sybil partielle (plafond de poids par validateur) ; pas de résistance formelle aux attaques 51 % ou long-range (PoS).
- Commit-reveal simplifié : le tirage combine hash de bloc et seed de validateur, mais reste manipulable par un validateur qui contrôle sa propre seed.
- Mempool local, non partagé entre nœuds.
- Pas d'audit de sécurité externe.

## 5. Travaux futurs

- VRF ou commit-reveal renforcé pour le tirage aléatoire.
- Propagation P2P du mempool.
- Découverte de pairs dynamique (gossip multi-hop).
- Audit de sécurité indépendant.

## 6. Avertissement

Projet expérimental et pédagogique. Non audité, non résistant aux attaques réseau réelles. Ne pas utiliser pour stocker de la valeur réelle.
