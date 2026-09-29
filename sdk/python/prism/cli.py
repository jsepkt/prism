"""
Prism Network Command-Line Interface (CLI)
"""

import sys
import argparse
try:
    from .client import PrismClient
    from .crypto import CryptoUtils
except (ImportError, ValueError):
    from prism.client import PrismClient
    from prism.crypto import CryptoUtils

def main():
    parser = argparse.ArgumentParser(
        prog="prism",
        description="Prism Network CLI - The Sovereign Context & Edge-AI Ledger",
    )
    parser.add_argument(
        "--rpc",
        default="http://127.0.0.1:8545",
        help="Node RPC URL (default: http://127.0.0.1:8545)",
    )

    subparsers = parser.add_subparsers(dest="command", help="Available commands")

    # Status / Health
    subparsers.add_parser("status", help="Check local node status and PoAC block height")

    # Keypair generator
    subparsers.add_parser("keygen", help="Generate a new sovereign Ed25519 keypair")

    # Faucet
    faucet_parser = subparsers.add_parser("faucet", help="Request testnet PRISM tokens from faucet")
    faucet_parser.add_argument("pubkey", help="Recipient public key hex")
    faucet_parser.add_argument("--amount", type=int, default=1000, help="Amount in PRISM (default 1000)")

    # Account Balance
    acc_parser = subparsers.add_parser("account", help="Check account balance by public key")
    acc_parser.add_argument("pubkey", help="Public key hex")

    # Bounties
    subparsers.add_parser("bounties", help="List active AI query bounties and cohort escrows")

    args = parser.parse_args()

    if not args.command:
        parser.print_help()
        sys.exit(0)

    client = PrismClient(rpc_url=args.rpc)

    try:
        if args.command == "status":
            health = client.get_health()
            print("\n[+] Prism Node Status:")
            print(f"    Consensus:   PoAC (Proof-of-Attested-Context)")
            print(f"    Status:      {health.status.upper()}")
            print(f"    Height:      #{health.block_height}")
            print(f"    Block Hash:  {health.latest_block_hash}")
            print(f"    Validator:   {health.validator}\n")

        elif args.command == "keygen":
            pub, priv = CryptoUtils.generate_keypair()
            print("\n[+] New Sovereign Keypair Generated:")
            print(f"    Public Key:  0x{pub}")
            print(f"    Private Key: 0x{priv}")
            print("    [!] Keep your private key safe and never share it!\n")

        elif args.command == "faucet":
            res = client.request_faucet(args.pubkey, amount=args.amount)
            print(f"\n[+] Faucet Drop Succeeded!")
            print(f"    Account:     0x{res.pubkey}")
            print(f"    New Balance: {res.new_balance:,} PRISM\n")

        elif args.command == "account":
            acc = client.get_account(args.pubkey)
            print(f"\n[+] Account Details:")
            print(f"    Public Key:  0x{acc.pubkey}")
            print(f"    Balance:     {acc.balance:,} PRISM")
            print(f"    Nonce:       {acc.nonce}\n")

        elif args.command == "bounties":
            bounties = client.get_bounties()
            print(f"\n[+] Registered AI Query Bounties ({len(bounties)}):")
            for b in bounties:
                print(f"    Bounty #{b.id}: Escrow {b.escrow_amount:,} PRISM (k-Threshold: {b.min_cohort_size})")
            print()

    except Exception as e:
        print(f"[-] Error: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
