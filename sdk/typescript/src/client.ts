import { Account, ContextSchema, NodeHealth, QueryBounty, Transaction } from './types';

export interface PrismClientConfig {
  rpcUrl?: string;
}

export class PrismClient {
  public readonly rpcUrl: string;

  constructor(config: PrismClientConfig = {}) {
    this.rpcUrl = config.rpcUrl || 'http://127.0.0.1:8545';
  }

  public async getHealth(): Promise<NodeHealth> {
    const res = await fetch(`${this.rpcUrl}/health`);
    if (!res.ok) throw new Error(`Health check failed: ${res.statusText}`);
    return res.json() as Promise<NodeHealth>;
  }

  public async getState(): Promise<any> {
    const res = await fetch(`${this.rpcUrl}/api/v1/state`);
    if (!res.ok) throw new Error(`Get state failed: ${res.statusText}`);
    return res.json();
  }

  public async getAccount(pubkey: string): Promise<Account> {
    const res = await fetch(`${this.rpcUrl}/api/v1/accounts/${pubkey}`);
    if (!res.ok) throw new Error(`Get account failed: ${res.statusText}`);
    return res.json() as Promise<Account>;
  }

  public async getBounties(): Promise<QueryBounty[]> {
    const res = await fetch(`${this.rpcUrl}/api/v1/bounties`);
    if (!res.ok) throw new Error(`Get bounties failed: ${res.statusText}`);
    return res.json() as Promise<QueryBounty[]>;
  }

  public async getSchemas(): Promise<ContextSchema[]> {
    const res = await fetch(`${this.rpcUrl}/api/v1/schemas`);
    if (!res.ok) throw new Error(`Get schemas failed: ${res.statusText}`);
    return res.json() as Promise<ContextSchema[]>;
  }

  public async submitTransaction(tx: Transaction): Promise<{ tx_hash: string }> {
    const res = await fetch(`${this.rpcUrl}/api/v1/transactions`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(tx),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: res.statusText }));
      throw new Error(`Transaction submission failed: ${JSON.stringify(err)}`);
    }
    return res.json() as Promise<{ tx_hash: string }>;
  }

  public async requestFaucet(pubkey: string, amount: number): Promise<{ pubkey: string; new_balance: number }> {
    const res = await fetch(`${this.rpcUrl}/api/v1/dev/faucet`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ pubkey, amount }),
    });
    if (!res.ok) throw new Error(`Faucet request failed: ${res.statusText}`);
    return res.json() as Promise<{ pubkey: string; new_balance: number }>;
  }
}
