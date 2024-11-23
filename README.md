# Drachma Blockchain

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Overview

**Drachma** is a blockchain project written in Rust, designed to prioritize security, modularity, and efficient performance. With features like Blake3 hashing, UTXO-based transaction management, and Merkle tree integration, Drachma lays the groundwork for a lightweight and scalable blockchain system.

Networking will leverage **QUIC** for efficient and reliable communication between nodes, ensuring low-latency and high-throughput peer-to-peer interactions.

While the **blockchain core is still under development**, several foundational components are already implemented.

## Current Status

### Completed
- **Block structure and logic**: Define and manage blocks.
- **Transaction models**: Create and validate transactions.
- **Merkle tree**: Ensure data integrity within blocks.
- **UTXO logic**: Manage unspent transaction outputs.
- **Transaction pool (Mempool)**: Temporarily store unconfirmed transactions.
- **Utility functions**: Provide reusable helper methods.

### Work in Progress
- **Blockchain core**: Building the chain logic and integration with other components.
- **QUIC Networking**: Implementing QUIC for peer-to-peer communication.

## Features

- **Efficient Hashing**: Powered by Blake3 for fast and secure block validation.
- **Modular Design**: Clear separation of concerns for maintainability.
- **Merkle Tree Integration**: Ensures data integrity for transactions within blocks.
- **UTXO Model**: Implements the Unspent Transaction Output model.
- **Transaction Pool**: Handles unconfirmed transactions before inclusion in blocks.
- **QUIC Networking**: High-performance, secure, and low-latency communication protocol.

## Project Structure

- **Cargo.toml**: Main project dependencies
- **corelib/**: Core library for blockchain logic
  - **src/**:
    - `block.rs`: Block structure and functionality
    - `transaction.rs`: Transaction model and logic
    - `merkle.rs`: Merkle tree implementation
    - `utxo.rs`: UTXO logic
    - `mempool.rs`: Transaction pool
    - `utils.rs`: Helper utilities
    - `errors.rs`: Error handling
    - `config.rs`: Configuration settings
- **node/**: Node application for P2P and consensus
  - **src/**:
    - `net.rs`: Networking logic (QUIC-based)
    - `node.rs`: Node management
- **wallet/**: Wallet application for end-user interaction (Planned)
- **target/**: Build artifacts

## Roadmap

- [x] Block structure and logic
- [x] Transaction models
- [x] Merkle tree for transaction integrity
- [x] UTXO-based transaction management
- [x] Transaction pool
- [ ] Blockchain core integration
- [ ] QUIC-based peer-to-peer networking
- [ ] Wallet application
- [ ] Consensus mechanisms
- [ ] Zero-Knowledge Proofs for privacy

## License

This project is licensed under the MIT License.
