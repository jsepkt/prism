import { SovereignVault } from './vault';
import { PrismClient } from './client';

async function main() {
  console.log('--- Testing Prism Sovereign Vault ---');
  const vault = new SovereignVault();

  vault.recordNumeric('health', 'weekly_run_distance_km', 24.5);
  vault.recordNumeric('health', 'avg_sleep_hours', 7.8);
  vault.recordString('lifestyle', 'diet', 'plant-forward');

  // Verify criteria check
  const criteria1 = {
    category: 'health',
    metric: 'weekly_run_distance_km',
    min_value: 15.0,
  };
  const match1 = vault.evaluateCriteria(criteria1);
  console.log(`Criteria 1 (>15km run): matched = ${match1} (expected: true)`);
  if (!match1) throw new Error('Criteria 1 should match');

  const criteria2 = {
    category: 'health',
    metric: 'weekly_run_distance_km',
    min_value: 40.0,
  };
  const match2 = vault.evaluateCriteria(criteria2);
  console.log(`Criteria 2 (>40km run): matched = ${match2} (expected: false)`);
  if (match2) throw new Error('Criteria 2 should not match');

  // Generate ZK proof
  const proof = vault.generateProof('RUNNER_SCHEMA_V1', criteria1, 0);
  console.log('Generated Proof:', proof ? 'Success' : 'Failed');
  if (!proof) throw new Error('Proof generation should succeed');
  console.log('Proof platform:', proof.hardwareAttestation.platform);

  console.log('\n--- Testing Prism Client Configuration ---');
  const client = new PrismClient({ rpcUrl: 'http://127.0.0.1:8545' });
  console.log('RPC Client initialized to:', client.rpcUrl);

  console.log('\n✅ All SDK Unit Tests Passed Successfully!');
}

main().catch((err) => {
  console.error('Test error:', err);
  process.exit(1);
});
