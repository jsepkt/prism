"""
Unit tests for Prism Python SDK
"""

import unittest
from prism.crypto import CryptoUtils, BUILDER_ROYALTY_BPS, DEFLATIONARY_BURN_BPS
from prism.models import QueryBounty, NodeHealth

class TestPrismSdk(unittest.TestCase):
    def test_fee_calculation(self):
        amount = 1_000_000
        fees = CryptoUtils.calculate_fees(amount)

        # 12,500 per million (1.25%)
        self.assertEqual(fees["builder_royalty"], 12_500)
        # 2,500 per million (0.25%)
        self.assertEqual(fees["deflationary_burn"], 2_500)
        # Total protocol fee = 15,000 (1.50%)
        self.assertEqual(fees["total_protocol_fee"], 15_000)
        # Users get 100% of promised principal
        self.assertEqual(fees["net_user_payout"], 1_000_000)

    def test_keypair_generation(self):
        pub, priv = CryptoUtils.generate_keypair()
        self.assertEqual(len(pub), 64)
        self.assertEqual(len(priv), 64)

    def test_model_instantiation(self):
        health = NodeHealth(
            status="healthy",
            block_height=42,
            latest_block_hash="0xabcd1234",
            validator="0xval5678",
            pending_mempool_txs=0,
        )
        self.assertEqual(health.block_height, 42)
        self.assertEqual(health.status, "healthy")

if __name__ == "__main__":
    unittest.main()
