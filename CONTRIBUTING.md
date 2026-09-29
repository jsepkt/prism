# Contributing to Prism Network

Thank you for your interest in contributing to **Prism Network**! Prism is an open-source, permissionless Layer-1 blockchain engineered to transform everyday human context into sovereign, encrypted, income-generating digital assets.

---

## 🛠️ Development Setup

### Prerequisites
* **Rust**: `1.90.0` or higher (`rustup update stable`)
* **Node.js**: `20.x` or `22.x` (for TypeScript SDK and client tooling)
* **Docker & Docker Compose**: (optional, for running local 3-node clusters)

### Building from Source
```bash
# Clone the repository
git clone https://github.com/jsepkt/prism.git
cd prism

# Build all workspace crates
cargo build

# Run the complete test suite
cargo test --workspace
```

---

## 🏗️ Repository Architecture

* `crates/prism-crypto`: BLAKE3, Ed25519, Zero-Knowledge circuits, Apple App Attest / Google Play Integrity.
* `crates/prism-core`: Merkle trees, state transitions, smart escrows, and the 12,500/M (1.25%) builder royalty engine.
* `crates/prism-consensus`: Proof-of-Attested-Context (PoAC) validator election engine.
* `crates/prism-client`: Sovereign on-device Context Vault.
* `crates/prism-p2p`: Async TCP wire framing protocol and gossip mesh.
* `crates/prism-agent`: Autonomous Intent Solvers and algorithmic bidding.
* `crates/prism-node`: Full node binary with embedded Axum REST / JSON-RPC server and Web3 dashboard.
* `sdk/typescript`: Official TypeScript SDK (`@prism-network/sdk`).
* `apps/mobile-native`: React Native Expo mobile application.

---

## 🧪 Testing Guidelines

Before opening a pull request:
1. Ensure all unit and integration tests pass:
   ```bash
   cargo test --workspace
   ```
2. Verify code formatting and linting:
   ```bash
   cargo fmt --check
   cargo clippy --workspace --all-targets -- -D warnings
   ```
3. Run the end-to-end economy simulation:
   ```bash
   cargo run --example simulated_economy -p prism-node
   ```

---

## 📜 Pull Request Process

1. Fork the repo and create a feature branch (`git checkout -b feat/my-improvement`).
2. Write clear, atomic commits using Conventional Commits (`feat:`, `fix:`, `docs:`, `perf:`).
3. Push to your branch and open a PR against the `main` branch of `jsepkt/prism`.
4. Automated GitHub Actions CI will run the test matrix across Linux, macOS, and Windows.
