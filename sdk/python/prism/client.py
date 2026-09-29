"""
Prism Network Python SDK - HTTP Client
"""

import urllib.request
import json
from typing import List, Dict, Any, Optional
from .models import (
    NodeHealth,
    Account,
    QueryBounty,
    ContextSchema,
    Transaction,
    FaucetResponse,
    SubmitTxResponse,
)

class PrismClient:
    """
    Client for interacting with a Prism Network node (PoAC Consensus).
    """

    def __init__(self, rpc_url: str = "http://127.0.0.1:8545"):
        self.rpc_url = rpc_url.rstrip("/")

    def _get(self, path: str) -> Dict[str, Any]:
        url = f"{self.rpc_url}{path}"
        req = urllib.request.Request(url, headers={"User-Agent": "Prism-Python-SDK/0.1.0"})
        with urllib.request.urlopen(req, timeout=10) as resp:
            data = resp.read().decode("utf-8")
            return json.loads(data)

    def _post(self, path: str, payload: Dict[str, Any]) -> Dict[str, Any]:
        url = f"{self.rpc_url}{path}"
        data_bytes = json.dumps(payload).encode("utf-8")
        req = urllib.request.Request(
            url,
            data=data_bytes,
            headers={
                "Content-Type": "application/json",
                "User-Agent": "Prism-Python-SDK/0.1.0",
            },
        )
        with urllib.request.urlopen(req, timeout=10) as resp:
            data = resp.read().decode("utf-8")
            return json.loads(data)

    def get_health(self) -> NodeHealth:
        """Fetch node health status, latest block height, and validator key."""
        res = self._get("/health")
        return NodeHealth(**res)

    def get_state(self) -> Dict[str, Any]:
        """Fetch complete current blockchain state."""
        return self._get("/api/v1/state")

    def get_account(self, pubkey: str) -> Account:
        """Fetch account balance and nonce by Ed25519 public key."""
        clean_pk = pubkey.replace("0x", "")
        res = self._get(f"/api/v1/accounts/{clean_pk}")
        if "pubkey" not in res:
            res["pubkey"] = clean_pk
        return Account(**res)

    def get_bounties(self) -> List[QueryBounty]:
        """Fetch all registered AI query bounties and active privacy cohorts."""
        res = self._get("/api/v1/bounties")
        return [QueryBounty(**b) for b in res]

    def get_schemas(self) -> List[ContextSchema]:
        """Fetch all registered on-chain context schemas."""
        res = self._get("/api/v1/schemas")
        return [ContextSchema(**s) for s in res]

    def request_faucet(self, pubkey: str, amount: int = 1000) -> FaucetResponse:
        """Request testnet PRISM tokens from the node faucet."""
        clean_pk = pubkey.replace("0x", "")
        res = self._post("/api/v1/dev/faucet", {"pubkey": clean_pk, "amount": amount})
        return FaucetResponse(**res)

    def submit_transaction(self, tx: Transaction) -> SubmitTxResponse:
        """Broadcast a signed transaction to the PoAC mempool."""
        res = self._post("/api/v1/transactions", tx.model_dump())
        return SubmitTxResponse(**res)
