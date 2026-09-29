import React, { useState, useEffect } from 'react';
import {
  StyleSheet,
  Text,
  View,
  TouchableOpacity,
  ScrollView,
  SafeAreaView,
  StatusBar,
  Alert,
} from 'react-native';
import { HealthBridge, DailyTelemetry } from './src/services/health';
import { SiliconEnclave } from './src/services/enclave';

export default function App() {
  const [telemetry, setTelemetry] = useState<DailyTelemetry | null>(null);
  const [unclaimedDividend, setUnclaimedDividend] = useState(142.50);
  const [unclaimedTokens, setUnclaimedTokens] = useState(950);
  const [isClaiming, setIsClaiming] = useState(false);

  useEffect(() => {
    loadHealthData();
  }, []);

  const loadHealthData = async () => {
    const data = await HealthBridge.getRollingTelemetry();
    setTelemetry(data);
  };

  const handleClaimDividend = async () => {
    setIsClaiming(true);
    try {
      const authenticated = await SiliconEnclave.authenticateBiometric(
        'Confirm FaceID to settle $142.50 (950 PRISM)'
      );

      if (!authenticated) {
        Alert.alert('Authentication Cancelled', 'Biometric validation required.');
        setIsClaiming(false);
        return;
      }

      // Generate Hardware Attestation
      const attestation = await SiliconEnclave.generateHardwareAttestation('bounty_marathon_01');

      // Settle
      setTimeout(() => {
        setUnclaimedDividend(0);
        setUnclaimedTokens(0);
        setIsClaiming(false);
        Alert.alert(
          '✓ Dividend Settled!',
          `950 PRISM deposited to your sovereign vault.\nAttestation ID: ${attestation.keyId.substring(0, 16)}...\nZero personal data leaked.`
        );
      }, 1200);
    } catch (e: any) {
      setIsClaiming(false);
      Alert.alert('Error', e.message || 'Settlement failed');
    }
  };

  return (
    <SafeAreaView style={styles.container}>
      <StatusBar barStyle="light-content" />
      <ScrollView contentContainerStyle={styles.scrollContent}>
        {/* Header */}
        <View style={styles.header}>
          <View>
            <Text style={styles.brandTitle}>PRISM VAULT</Text>
            <Text style={styles.brandSubtitle}>SOVEREIGN LIVING DIVIDEND</Text>
          </View>
          <View style={styles.statusPill}>
            <View style={styles.pulseDot} />
            <Text style={styles.statusText}>ENCLAVE ACTIVE</Text>
          </View>
        </View>

        {/* Hero Card */}
        <View style={styles.heroCard}>
          <Text style={styles.heroLabel}>UNCLAIMED LIVING DIVIDEND</Text>
          <View style={styles.balanceRow}>
            <Text style={styles.fiatAmount}>${unclaimedDividend.toFixed(2)}</Text>
            <View style={styles.tokenPill}>
              <Text style={styles.tokenText}>{unclaimedTokens} PRISM</Text>
            </View>
          </View>

          <View style={styles.kpiRow}>
            <View style={styles.kpiCell}>
              <Text style={styles.kpiVal}>14/14</Text>
              <Text style={styles.kpiLbl}>Days Active</Text>
            </View>
            <View style={styles.kpiCell}>
              <Text style={styles.kpiVal}>37</Text>
              <Text style={styles.kpiLbl}>ZK Queries</Text>
            </View>
            <View style={styles.kpiCell}>
              <Text style={[styles.kpiVal, { color: '#34d399' }]}>0 bytes</Text>
              <Text style={styles.kpiLbl}>Data Leaked</Text>
            </View>
          </View>

          <TouchableOpacity
            style={[styles.claimButton, isClaiming && { opacity: 0.7 }]}
            onPress={handleClaimDividend}
            disabled={isClaiming || unclaimedTokens === 0}
          >
            <Text style={styles.claimButtonText}>
              {isClaiming ? 'Verifying FaceID & Generating Proof...' : 'Claim Daily Living Dividend'}
            </Text>
          </TouchableOpacity>
        </View>

        {/* Telemetry Stack */}
        <Text style={styles.sectionTitle}>VERIFIED CONTEXT VAULT</Text>

        <View style={styles.telemetryCard}>
          <View style={styles.cardLeft}>
            <View style={[styles.orbIcon, { backgroundColor: '#10b981' }]} />
            <View>
              <Text style={styles.cardTitle}>Cardio & Distance</Text>
              <Text style={styles.cardSubtitle}>Criteria &gt;= 15.0 km/wk</Text>
            </View>
          </View>
          <View style={styles.cardRight}>
            <Text style={styles.cardValue}>{telemetry?.distanceKm || 0} km</Text>
            <Text style={styles.verifiedTag}>✓ Verified</Text>
          </View>
        </View>

        <View style={styles.telemetryCard}>
          <View style={styles.cardLeft}>
            <View style={[styles.orbIcon, { backgroundColor: '#8b5cf6' }]} />
            <View>
              <Text style={styles.cardTitle}>Sleep & Recovery</Text>
              <Text style={styles.cardSubtitle}>Criteria &gt;= 6.5 hrs/day</Text>
            </View>
          </View>
          <View style={styles.cardRight}>
            <Text style={styles.cardValue}>{telemetry?.sleepHours || 0} hrs</Text>
            <Text style={styles.verifiedTag}>✓ Verified</Text>
          </View>
        </View>

        <View style={styles.telemetryCard}>
          <View style={styles.cardLeft}>
            <View style={[styles.orbIcon, { backgroundColor: '#06b6d4' }]} />
            <View>
              <Text style={styles.cardTitle}>Focus & Screen</Text>
              <Text style={styles.cardSubtitle}>Screen &lt; 4.0 hrs/day</Text>
            </View>
          </View>
          <View style={styles.cardRight}>
            <Text style={styles.cardValue}>{telemetry?.screenTimeHours || 0} hrs</Text>
            <Text style={styles.verifiedTag}>✓ Verified</Text>
          </View>
        </View>

        <View style={styles.telemetryCard}>
          <View style={styles.cardLeft}>
            <View style={[styles.orbIcon, { backgroundColor: '#f59e0b' }]} />
            <View>
              <Text style={styles.cardTitle}>Intent Solvers</Text>
              <Text style={styles.cardSubtitle}>Zero-Middleman Deals</Text>
            </View>
          </View>
          <View style={styles.cardRight}>
            <Text style={[styles.cardValue, { color: '#fbbf24' }]}>3 Bids</Text>
            <Text style={[styles.verifiedTag, { color: '#38bdf8' }]}>Competing</Text>
          </View>
        </View>

        {/* Silicon Notice */}
        <View style={styles.siliconNotice}>
          <Text style={styles.siliconTitle}>Hardware Silicon Enclave</Text>
          <Text style={styles.siliconText}>
            Protected by Apple App Attest. Payouts are locked until at least 500 cohort peers qualify.
          </Text>
        </View>
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#030712',
  },
  scrollContent: {
    padding: 16,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 16,
  },
  brandTitle: {
    color: '#ffffff',
    fontSize: 20,
    fontWeight: '800',
    letterSpacing: -0.5,
  },
  brandSubtitle: {
    color: '#06b6d4',
    fontSize: 10,
    fontWeight: '700',
    letterSpacing: 1,
  },
  statusPill: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: 'rgba(16, 185, 129, 0.15)',
    borderWidth: 1,
    borderColor: 'rgba(16, 185, 129, 0.3)',
    borderRadius: 999,
    paddingHorizontal: 10,
    paddingVertical: 4,
  },
  pulseDot: {
    width: 6,
    height: 6,
    borderRadius: 3,
    backgroundColor: '#34d399',
    marginRight: 6,
  },
  statusText: {
    color: '#34d399',
    fontSize: 11,
    fontWeight: '700',
  },
  heroCard: {
    backgroundColor: '#0f172a',
    borderRadius: 24,
    borderWidth: 1,
    borderColor: 'rgba(255, 255, 255, 0.1)',
    padding: 20,
    marginBottom: 20,
  },
  heroLabel: {
    color: '#94a3b8',
    fontSize: 11,
    fontWeight: '700',
    letterSpacing: 1,
  },
  balanceRow: {
    flexDirection: 'row',
    alignItems: 'baseline',
    marginVertical: 10,
  },
  fiatAmount: {
    color: '#ffffff',
    fontSize: 38,
    fontWeight: '800',
    marginRight: 10,
  },
  tokenPill: {
    backgroundColor: 'rgba(99, 102, 241, 0.2)',
    borderWidth: 1,
    borderColor: 'rgba(99, 102, 241, 0.4)',
    borderRadius: 12,
    paddingHorizontal: 8,
    paddingVertical: 4,
  },
  tokenText: {
    color: '#a5b4fc',
    fontSize: 13,
    fontWeight: '700',
  },
  kpiRow: {
    flexDirection: 'row',
    justifyContent: 'space-around',
    borderTopWidth: 1,
    borderTopColor: 'rgba(255, 255, 255, 0.08)',
    paddingTop: 12,
    marginBottom: 16,
  },
  kpiCell: {
    alignItems: 'center',
  },
  kpiVal: {
    color: '#ffffff',
    fontSize: 15,
    fontWeight: '800',
  },
  kpiLbl: {
    color: '#94a3b8',
    fontSize: 10,
  },
  claimButton: {
    backgroundColor: '#4f46e5',
    borderRadius: 16,
    paddingVertical: 14,
    alignItems: 'center',
  },
  claimButtonText: {
    color: '#ffffff',
    fontSize: 15,
    fontWeight: '700',
  },
  sectionTitle: {
    color: '#94a3b8',
    fontSize: 12,
    fontWeight: '800',
    letterSpacing: 1,
    marginBottom: 10,
  },
  telemetryCard: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    backgroundColor: 'rgba(15, 23, 42, 0.7)',
    borderWidth: 1,
    borderColor: 'rgba(255, 255, 255, 0.08)',
    borderRadius: 18,
    padding: 14,
    marginBottom: 10,
  },
  cardLeft: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  orbIcon: {
    width: 38,
    height: 38,
    borderRadius: 12,
    marginRight: 12,
  },
  cardTitle: {
    color: '#ffffff',
    fontSize: 14,
    fontWeight: '700',
  },
  cardSubtitle: {
    color: '#94a3b8',
    fontSize: 11,
  },
  cardRight: {
    alignItems: 'flex-end',
  },
  cardValue: {
    color: '#ffffff',
    fontSize: 15,
    fontWeight: '800',
  },
  verifiedTag: {
    color: '#34d399',
    fontSize: 11,
    fontWeight: '600',
  },
  siliconNotice: {
    backgroundColor: 'rgba(99, 102, 241, 0.08)',
    borderWidth: 1,
    borderColor: 'rgba(99, 102, 241, 0.25)',
    borderRadius: 18,
    padding: 14,
    marginTop: 10,
  },
  siliconTitle: {
    color: '#ffffff',
    fontSize: 13,
    fontWeight: '700',
  },
  siliconText: {
    color: '#94a3b8',
    fontSize: 11,
    marginTop: 4,
    lineHeight: 16,
  },
});
