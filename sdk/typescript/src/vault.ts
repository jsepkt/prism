import { HardwareAttestation, NumericCriteria, ZkProof } from './types';

export class SovereignVault {
  private records: Map<string, number | string> = new Map();

  /**
   * Record a numeric metric (e.g. from Apple Health or Google Fit)
   */
  public recordNumeric(category: string, metric: string, value: number): void {
    this.records.set(`${category}:${metric}`, value);
  }

  /**
   * Record a categorical metric (e.g. dietary or airline preference)
   */
  public recordString(category: string, metric: string, value: string): void {
    this.records.set(`${category}:${metric}`, value);
  }

  public getNumeric(category: string, metric: string): number | undefined {
    const val = this.records.get(`${category}:${metric}`);
    return typeof val === 'number' ? val : undefined;
  }

  public getString(category: string, metric: string): string | undefined {
    const val = this.records.get(`${category}:${metric}`);
    return typeof val === 'string' ? val : undefined;
  }

  /**
   * Evaluates criteria against the local vault. 0 bytes leave the device if false.
   */
  public evaluateCriteria(criteria: NumericCriteria): boolean {
    const val = this.getNumeric(criteria.category, criteria.metric);
    if (val === undefined) return false;
    if (criteria.min_value !== undefined && val < criteria.min_value) return false;
    if (criteria.max_value !== undefined && val > criteria.max_value) return false;
    return true;
  }

  /**
   * Generate an edge Zero-Knowledge proof and hardware attestation if criteria is met
   */
  public generateProof(
    schemaId: string,
    criteria: NumericCriteria,
    accountNonce: number
  ): { zkProof: ZkProof; hardwareAttestation: HardwareAttestation } | null {
    if (!this.evaluateCriteria(criteria)) {
      return null;
    }

    const zkProof: ZkProof = {
      circuit_id: schemaId,
      public_inputs: Array.from(new TextEncoder().encode(JSON.stringify(criteria))),
      proof_bytes: [1, 2, 3, 4, 5],
    };

    const hardwareAttestation: HardwareAttestation = {
      platform: 'DevnetMock',
      device_root_hash: '00'.repeat(32),
      nonce: accountNonce,
      attestation_token: new Array(32).fill(42),
    };

    return { zkProof, hardwareAttestation };
  }
}
