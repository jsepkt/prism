"""
Prism Network Python SDK - Data Models
"""

from typing import List, Optional, Dict, Any
from pydantic import BaseModel, Field

class NodeHealth(BaseModel):
    status: str
    block_height: int
    latest_block_hash: str
    validator: str
    pending_mempool_txs: int

class Account(BaseModel):
    pubkey: Optional[str] = None
    balance: int
    nonce: int
    reputation: Optional[int] = 100
    is_hardware_attested: Optional[bool] = False
    verified_proofs_count: Optional[int] = 0

class QueryBounty(BaseModel):
    id: str
    sponsor: str
    schema_id: str
    escrow_amount: int
    min_cohort_size: int
    reward_per_participant: int
    qualified_participants: List[str] = Field(default_factory=list)
    unlocked: bool = False

class ContextSchema(BaseModel):
    id: str
    name: str
    description: str
    circuit_verification_key: str
    reward_amount: int

class Transaction(BaseModel):
    sender: str
    recipient: str
    amount: int
    nonce: int
    signature: str
    payload_type: str = "transfer"
    data: Optional[Dict[str, Any]] = None

class FaucetResponse(BaseModel):
    pubkey: str
    new_balance: int

class SubmitTxResponse(BaseModel):
    tx_hash: str
