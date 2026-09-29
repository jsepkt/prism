# Prism Network - Python SDK & CLI

The official Python client library and developer CLI for **Prism Network** — the Layer-1 Zero-Knowledge Edge-AI Ledger powered by **Proof-of-Attested-Context (PoAC)**.

Designed for AI research labs, autonomous agent builders, and data science pipelines to query privacy-preserving decentralized cohorts, verify zero-knowledge enclave proofs, and automate micro-incentive bounties.

---

## ⚡ Features

- **🚀 PoAC Node RPC Client**: Native HTTP client for interacting with any local or remote Prism validator node.
- **🛡️ Ed25519 Cryptographic Signatures**: Autonomous agent keypair generation, public key encoding, and cryptographically verified transaction signing.
- **🧠 AI Query Bounties & Privacy Cohorts**: Register context schemas, fund escrow bounties for confidential user cohorts, and trigger zero-knowledge threshold releases.
- **💎 Enshrined Builder Royalties**: Fully compatible with Prism's 1.25% (125 BPS) builder royalty protocol fee model.
- **💻 Developer CLI (`prism`)**: Built-in CLI tool to inspect node status, generate keys, query accounts, and request testnet drops.

---

## 📦 Installation

```bash
# Clone the repository
git clone https://github.com/jsepkt/prism.git
cd prism/sdk/python

# Install in development mode
pip install -e .
```

### Requirements
- Python `>= 3.9`
- `pydantic >= 2.0`
- `cryptography >= 41.0.0`

---

## 🛠️ CLI Quickstart

Once installed, the `prism` command is available in your terminal:

```bash
# 1. Check local validator status & PoAC block height
prism status

# 2. Generate a new sovereign Ed25519 agent keypair
prism keygen

# 3. Request 1,000 testnet PRISM tokens from the dev faucet
prism faucet <PUBLIC_KEY_HEX>

# 4. Check on-chain account balance and nonce
prism account <PUBLIC_KEY_HEX>

# 5. List active AI query bounties and cohort escrows
prism bounties
```

Specify a custom RPC endpoint with `--rpc`:
```bash
prism --rpc http://testnet.prism.network:8545 status
```

---

## 🐍 Python API Usage

### 1. Initialize Client & Inspect Ledger

```python
from prism import PrismClient

client = PrismClient(rpc_url="http://127.0.0.1:8545")

# Check node health
health = client.get_health()
print(f"Current PoAC Block Height: #{health.block_height}")
print(f"Active Validator: {health.validator}")
```

### 2. Generate Agent Keypair & Sign Transactions

```python
from prism import CryptoUtils, Transaction

# Generate sovereign keypair for an autonomous agent
pubkey, privkey = CryptoUtils.generate_keypair()
print(f"Agent Pubkey: 0x{pubkey}")

# Prepare transaction payload
sender = pubkey
recipient = "0000000000000000000000000000000000000000000000000000000000000001"
amount = 500
nonce = 0

# Sign payload with private key
msg = f"{sender}:{recipient}:{amount}:{nonce}".encode("utf-8")
signature = CryptoUtils.sign_message(privkey, msg)

tx = Transaction(
    sender=sender,
    recipient=recipient,
    amount=amount,
    nonce=nonce,
    signature=signature,
    payload_type="transfer"
)

# Submit to PoAC mempool
res = client.submit_transaction(tx)
print(f"Transaction submitted! Hash: {res.tx_hash}")
```

### 3. AI Query Bounties & Privacy-Preserving Cohorts

```python
# Query active AI bounties
bounties = client.get_bounties()
for b in bounties:
    print(f"Bounty #{b.id}:")
    print(f"  Sponsor: {b.sponsor}")
    print(f"  Schema:  {b.schema_id}")
    print(f"  Escrow:  {b.escrow_amount:,} PRISM")
    print(f"  Target Cohort Size (k): {b.min_cohort_size}")
    print(f"  Qualified Edge Devices: {len(b.qualified_participants)}")
```

---

## 🔬 Autonomous AI Training Pipeline Example

A complete end-to-end example demonstrating how a research lab funds an AI bounty, discovers private cohorts, and verifies proof settlements is included in [`examples/ai_training_pipeline.py`](examples/ai_training_pipeline.py):

```bash
python examples/ai_training_pipeline.py
```

---

## 🧪 Running Tests

```bash
cd sdk/python
python -m unittest discover tests
```

---

## 📄 License

Licensed under the MIT License. Built with ❤️ for the decentralized AI and privacy ecosystem by [jsepkt](https://github.com/jsepkt).
