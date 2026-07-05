# Quark (QRK)

Cryptomonnaie expérimentale en Rust : Proof-of-Stake + système de collection Quark/Hadron.

Voir [WHITEPAPER.md](./WHITEPAPER.md) pour l'architecture détaillée.

## Installation

```bash
git clone https://github.com/Zeemoud/Quark
cd Quark
cargo build
```

## Lancer un nœud

```bash
cargo run
```

Démarre :

- un nœud P2P sur le port `8080`
- une API HTTP sur le port `3000` (`/chain`, `/peers`, `/`)
- un menu interactif en ligne de commande

## Menu CLI

| Option | Action                                     |
| ------ | ------------------------------------------ |
| 1      | Créer un wallet (chiffré par mot de passe) |
| 2      | Voir le solde et les Quarks d'une adresse  |
| 3      | Envoyer une transaction                    |
| 4      | Forger un bloc                             |
| 5      | Combiner 3 Quarks en Hadron                |
| 6      | Quitter (sauvegarde la chain)              |
| 7      | Synchroniser avec un pair                  |

## Tester plusieurs nœuds en local

```bash
# Terminal 1
cargo run

# Terminal 2 (changer le port dans le code : 8081)
cargo run --target-dir target2
```

## Tests

```bash
cargo test
```

## Structure du projet

```
src/
  main.rs        menu CLI
  block.rs       Block, Blockchain
  transaction.rs Transaction
  wallet.rs      Wallet (clés, chiffrement)
  quark.rs       QuarkType, Hadron, forge_hadron
  validator.rs   Validator, select_validator (PoS)
  ledger.rs      Ledger (soldes)
  network.rs     P2P, API HTTP
```

## Avertissement

Projet expérimental et pédagogique, non audité. Ne pas utiliser pour stocker de la valeur réelle.
