# Prism Network: Public Launch & Ecosystem Grant Kit

This kit contains ready-to-publish launch announcements, social posts, and ecosystem grant applications to introduce Prism Network to the world.

---

## 1. X (Twitter) Launch Announcement Thread

**Tweet 1 (The Hook):**
> Bitcoin gave humanity sovereign money.
> 
> Today, I'm open-sourcing **Prism Network**: a next-generation Layer-1 blockchain that turns everyday human digital activity into sovereign, encrypted, income-generating digital assets using Zero-Knowledge Edge AI.
> 
> 100% open source on GitHub: https://github.com/jsepkt/prism 🧵👇

**Tweet 2 (The Problem):**
> Big Tech extracts trillions of dollars in value from your health, browsing, and behavioral habits. In exchange? You get targeted ads and 0% of the profits.
> 
> Meanwhile, frontier AI labs are starved for high-quality, authentic human context data to align models.

**Tweet 3 (The Prism Solution):**
> Prism solves this through **Proof-of-Attested-Context (PoAC)**:
> 
> 📱 Your phone runs small language models (SLMs) locally inside your Secure Enclave (Apple App Attest / Android Play Integrity).
> 🔒 Raw data never leaves your device (GDPR & HIPAA exempt).
> ⚡ Only mathematical Zero-Knowledge proofs hit the ledger.

**Tweet 4 (The Daily Living Dividend):**
> Everyday people earn a passive "Daily Living Dividend" ($50–$150/mo) in $PRISM simply by letting their device verify research queries without surveillance.
> 
> Smart-contract enforced $k$-anonymity ($k \ge 500$) guarantees payouts are cryptographically locked until a full privacy cohort submits proofs.

**Tweet 5 (Autonomous Intent Commerce):**
> We've also built an Autonomous Intent Clearinghouse (`crates/prism-agent`).
> 
> AI agents broadcast purchasing intents (e.g. "Find direct flight to Tokyo under $750"), and competing algorithmic solvers bid down prices in real-time. Zero middleman markups.

**Tweet 6 (Tokenomics & Builder Royalty):**
> Tokenomics are enshrined in code:
> • 100% of promised data dividends go to everyday users.
> • 1.25% (12,500 per million) builder protocol royalty funds ongoing open-source R&D.
> • 0.25% (25 BPS) of every transaction is permanently burned from supply.

**Tweet 7 (Call to Action):**
> 📖 Whitepaper: https://github.com/jsepkt/prism/blob/main/WHITEPAPER.md
> 🖥️ Web Explorer & Simulator: https://jsepkt.github.io/prism/
> 📦 TypeScript SDK: `@prism-network/sdk`
> 
> Star the repo on GitHub, spin up a local validator node with `cargo run -p prism-node`, and let's build a sovereign internet together! 🌟

---

## 2. Hacker News: Show HN Post

**Title:** Show HN: Prism – Open-source L1 blockchain for ZK Edge-AI and data dividends (Rust)

**Post Body:**
> Hi HN,
> 
> I built Prism Network (https://github.com/jsepkt/prism), an open-source Layer-1 blockchain written in Rust that turns everyday personal telemetry into zero-knowledge attested computational assets.
> 
> ### The Core Problem
> Today, frontier AI models desperately need authentic human telemetry (wearables, sleep patterns, real-world context) to advance personal AI assistants. However, centralized scraping creates massive privacy violations and regulatory nightmares (GDPR/HIPAA), while everyday consumers receive zero compensation for their data.
> 
> ### How Prism Works
> 1. **On-Device Sovereign Context Vault (`crates/prism-client`)**: Telemetry remains encrypted strictly inside the user's local device sandbox.
> 2. **Hardware Attestation (`crates/prism-crypto`)**: Cryptographic nonces are signed by hardware keystores (Apple App Attest DCAppAttest and Google Play Integrity) to prevent bot-farm sybil attacks.
> 3. **Zero-Knowledge Validity Proofs (zk-CP)**: The device verifies constraints locally and submits a Groth16/Circom-compatible zero-knowledge proof to the ledger.
> 4. **Smart-Contract Enforced $k$-Anonymity (`crates/prism-core`)**: Payouts from enterprise query escrows remain cryptographically locked until at least $k$ distinct attested nodes submit proofs ($k \ge 3$ on devnet, $k \ge 500$ on mainnet), mathematically preventing de-anonymization.
> 5. **PoAC Consensus (`crates/prism-consensus`)**: Block proposers are elected based on attested proof verification throughput rather than wasteful hash power.
> 6. **Autonomous Intent Clearinghouse (`crates/prism-agent`)**: A decentralized solver auction where consumer purchasing requests are matched by competing algorithmic agents.
> 
> ### Tech Stack
> * Core Engine: Rust (Tokio, Axum, Ed25519, BLAKE3, Serde)
> * UI: Dark-glassmorphic dashboard with real-time HTML5 canvas particle physics and embedded RPC console.
> * SDK: TypeScript / Node.js client (`@prism-network/sdk`)
> 
> You can test the web simulator at https://jsepkt.github.io/prism/ or run a local node with `cargo run -p prism-node`.
> 
> Would love your feedback on the architecture, consensus design, and privacy cohort math!

---

## 3. Reddit (r/rust & r/crypto) Post

**Title:** I built an open-source Layer-1 blockchain in Rust with ZK Edge-AI and on-device privacy proofs

**Post Body:**
> Hey everyone,
> 
> Over the last several months, I've been architecting a Rust Layer-1 blockchain called Prism Network (https://github.com/jsepkt/prism).
> 
> The premise is simple: rather than designing blockchains exclusively for DeFi speculation or NFT mints, Prism allows everyday people to earn micro-dividends ($50–$150/mo) by running zero-knowledge queries on their personal smartphone telemetry without ever exposing raw data to the cloud.
> 
> What's implemented in the repo:
> * `crates/prism-crypto`: BLAKE3, Ed25519, Hardware Attestation (Apple App Attest / Android Integrity), ZK circuits.
> * `crates/prism-core`: Merkle roots, block structures, state transition engine with smart escrows, and a 1.25% protocol builder royalty + 0.25% burn.
> * `crates/prism-consensus`: Proof-of-Attested-Context (PoAC) validator election.
> * `crates/prism-client`: Local Sovereign Vault for evaluating criteria on edge devices.
> * `crates/prism-agent`: Autonomous intent solver order book.
> * `crates/prism-p2p`: Async TCP framing and gossip protocol.
> * `crates/prism-node`: Axum REST/JSON-RPC node with an embedded dashboard.
> 
> Full Whitepaper and code are open-source under MIT/Apache-2.0. Looking for Rustaceans to review the codebase and test validator clustering!

---

## 4. Web3 Foundation & Grant Application Template

**Target Programs**: Gitcoin Grants, Protocol Labs, Arbitrum Foundation, Optimism RetroPGF, Solana Foundation, Web3 Foundation.

### Project Overview
* **Project Name**: Prism Network
* **Category**: Privacy, Zero-Knowledge Proofs, Edge AI & Decentralized Infrastructure (DePIN)
* **Lead Builder**: jsepkt (`https://github.com/jsepkt`)
* **Repository**: https://github.com/jsepkt/prism
* **Live Demo**: https://jsepkt.github.io/prism/
* **License**: Open Source (Dual MIT / Apache-2.0)

### Executive Summary
Prism Network is a decentralized Layer-1 ledger purpose-built for the Edge-AI era. By combining hardware-backed attestation (Apple App Attest / Google Play Integrity) with Zero-Knowledge Context Proofs (zk-CP), Prism enables privacy-preserving consumer data dividends while strictly enforcing $k$-anonymity guarantees on-chain.

### What Makes Prism Unique?
1. **Solves the AI Data Bottleneck Privately**: Provides high-integrity human telemetry for model training and brand intelligence without violating GDPR/HIPAA.
2. **True Consumer Utility**: Shifts crypto adoption from niche financial trading to everyday users earning passive income on their mobile devices.
3. **Production Rust Architecture**: Clean modular crates, 100% test coverage, TypeScript SDK, and multi-node Docker orchestration.

### Requested Funding & Milestone Breakdown
* **Milestone 1 ($15,000)**: Mobile Native SDK (Swift / Kotlin bindings for on-device Circom proof generation and Secure Enclave signing).
* **Milestone 2 ($20,000)**: Multi-region Public Testnet with 20 independent validator nodes and block explorer indexer.
* **Milestone 3 ($15,000)**: Security Audit of cryptographic primitives, zero-knowledge circuits, and cohort escrow smart contracts.
