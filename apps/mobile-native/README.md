# Prism Vault Native App (iOS & Android)

This directory contains the production React Native / Expo application for everyday smartphone users.

---

## 🌟 Key Features

1. **Apple HealthKit & Google Health Connect Integration**:
   - Reads 7-day rolling cardio distance, resting heart rate, and sleep duration.
   - Evaluates criteria locally on the device with zero cloud telemetry leakage.
2. **Biometric Authorization (FaceID / Fingerprint)**:
   - Hardware-level confirmation using `expo-local-authentication`.
3. **Hardware-Backed Device Attestation**:
   - Apple App Attest (`DCAppAttestService`) and Android Play Integrity.
4. **Zero-Knowledge Proof Prover**:
   - Integrates with `@prism-network/sdk` to submit Groth16 cryptographic validity proofs directly to the Prism blockchain ledger.

---

## 🚀 Running the App Locally

### 1. Install Dependencies
```bash
cd apps/mobile-native
npm install
```

### 2. Launch Development Server
```bash
npx expo start
```
* Press **`i`** to launch in the **iOS Simulator** (requires macOS with Xcode).
* Press **`a`** to launch in the **Android Emulator** (Android Studio).
* Scan the QR code with the **Expo Go** app on your physical iPhone or Android phone.

---

## 📦 Building Standalone Binaries (EAS Build)

To build native `.ipa` (iOS) and `.apk` / `.aab` (Android) binaries for distribution on the Apple App Store / TestFlight and Google Play Store:

```bash
# Install EAS CLI
npm install -g eas-cli

# Login and build for iOS
eas build --platform ios

# Build for Android
eas build --platform android
```
