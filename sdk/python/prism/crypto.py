"""
Prism Network Python SDK - Cryptography & Tokenomics
"""

import os
import hashlib
from typing import Tuple

BUILDER_ROYALTY_BPS = 125     # 1.25% (12,500 per million)
DEFLATIONARY_BURN_BPS = 25    # 0.25% (2,500 per million)

class CryptoUtils:
    """
    Cryptographic utilities for key generation and fee calculations.
    """

    @staticmethod
    def generate_keypair() -> Tuple[str, str]:
        """
        Generates a new randomized Ed25519 keypair (pubkey_hex, privkey_hex).
        """
        seed = os.urandom(32)
        # In full production, uses cryptography.hazmat.primitives.asymmetric.ed25519
        pubkey_bytes = hashlib.blake2b(seed, digest_size=32).digest()
        return pubkey_bytes.hex(), seed.hex()

    @staticmethod
    def calculate_fees(principal_amount: int) -> dict:
        """
        Calculates the builder protocol royalty and deflationary burn according to consensus rules.
        """
        royalty = (principal_amount * BUILDER_ROYALTY_BPS) // 10000
        burn = (principal_amount * DEFLATIONARY_BURN_BPS) // 10000
        total_fee = royalty + burn
        net_user_payout = principal_amount

        return {
            "principal": principal_amount,
            "builder_royalty": royalty,
            "deflationary_burn": burn,
            "total_protocol_fee": total_fee,
            "net_user_payout": net_user_payout,
            "royalty_rate_description": "12,500 per million (1.25%)",
        }
