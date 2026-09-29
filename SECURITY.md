# Security Policy

The Prism Network community takes protocol security, cryptographic integrity, and zero-knowledge circuit correctness with the utmost seriousness.

---

## 🛡️ Supported Versions

We provide security patches for the latest active release branch:

| Version | Supported |
| :--- | :--- |
| `0.1.x` (Mainnet Alpha / Testnet) | ✅ Supported |
| `< 0.1.0` | ❌ Not Supported |

---

## 🚨 Reporting a Vulnerability

If you discover a security vulnerability within the Prism Network protocol, cryptographic circuits, smart contracts, or client SDK:

1. **Do NOT open a public GitHub issue.** Public disclosure puts network participants and validator nodes at risk.
2. Submit your report directly via **GitHub Private Security Advisory** on the repository:
   - Go to [https://github.com/jsepkt/prism/security/advisories/new](https://github.com/jsepkt/prism/security/advisories/new)
3. Alternatively, email the security team at **`security@prism-network.org`**.

### What to Include
* A detailed description of the vulnerability.
* Steps to reproduce or proof-of-concept (PoC) code.
* Potential impact on consensus state, cryptographic proofs, or funds.

---

## 🏆 Bug Bounty Program

We recognize and reward ethical security researchers who help protect the protocol. Bounties are awarded based on severity (CVSS):
* **Critical** (Consensus halt, state forgery, double-spend): Up to $50,000 in PRISM / USDC
* **High** (Proof forgery, Sybil circumvention): Up to $20,000 in PRISM / USDC
* **Medium** (P2P gossip amplification, local DoS): Up to $5,000 in PRISM / USDC
