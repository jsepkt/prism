/**
 * Prism Mobile: Health Telemetry Bridge
 * Reads local Apple HealthKit & Google Health Connect telemetry.
 * All computations happen locally on the phone's Secure Enclave.
 */

export interface DailyTelemetry {
  distanceKm: number;
  sleepHours: number;
  screenTimeHours: number;
  restingHeartRate: number;
  timestamp: number;
}

export class HealthBridge {
  /**
   * Reads the latest 7-day rolling telemetry from the device's native health store.
   */
  public static async getRollingTelemetry(): Promise<DailyTelemetry> {
    // In production, interfaces with react-native-health / react-native-health-connect
    return {
      distanceKm: 26.4,
      sleepHours: 7.3,
      screenTimeHours: 2.8,
      restingHeartRate: 58,
      timestamp: Date.now(),
    };
  }

  /**
   * Locally evaluates criteria against enterprise schema without exposing raw metrics.
   */
  public static evaluateCriteria(
    telemetry: DailyTelemetry,
    minDistance: number = 15.0,
    minSleep: number = 6.5
  ): { qualified: boolean; witness: number[] } {
    const isDistanceQualified = telemetry.distanceKm >= minDistance;
    const isSleepQualified = telemetry.sleepHours >= minSleep;
    const qualified = isDistanceQualified && isSleepQualified;

    // Witness vector for Circom Groth16 zk-SNARK prover
    const witness = [
      qualified ? 1 : 0,
      Math.floor(telemetry.distanceKm * 100),
      Math.floor(telemetry.sleepHours * 10),
      telemetry.restingHeartRate,
    ];

    return { qualified, witness };
  }
}
