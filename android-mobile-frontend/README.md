# android-mobile-frontend

Native Kotlin/Jetpack Compose client for Android phones. It supports seeded creator login, public browse/search/infinite feed, MP4 upload through a presigned URL, and Media3 HLS playback.

> Documentation: [project index](../docs/README.md) · [repository overview](../README.md)

Use the checked-in Gradle wrapper: `./gradlew testDebugUnitTest` for JVM tests and `./gradlew createDebugUnitTestCoverageReport createDebugCoverageReport` for unit and emulator coverage reports. The emulator suite uses the loopback-only login/storage adapter at `scripts/android-test-identity-server.py` plus the local synthetic feed/HLS endpoint described in [the verification record](../docs/verification/README.md). Forward those local ports with `adb reverse`. For local Kind access, also run `adb reverse` for the web API and MinIO upload ports.

## Component ownership, prerequisites, and lifecycle

- **Owner:** `opentube` / `android-mobile-frontend`.
- **API, event, and data contract owners/producers/consumers:** see the [contract catalog](../docs/contracts/README.md) for each authoritative interface.
- **Parent architecture:** [System Design](../docs/system-design.md).

### Build prerequisites

Use this component’s pinned toolchain and lockfile/wrapper. The supported versions and complete local build environment are listed in the [root README](../README.md).

### Use prerequisites

This component is used as part of the parent system. Start its required local dependencies and use the supported local access path described in the [root README](../README.md).

### Build, verify, deploy, undeploy, and use

Build, run, and verification commands for this component are documented above. There is no independent release lifecycle for this component.
The parent Helm release owns deployment and removal; follow the [chart guide](../deploy/helm/opentube/README.md) and [root deployment lifecycle](../README.md). Uninstall removes application workloads while retained PVCs keep their data; deleting the namespace or claims purges persistent data.

[Documentation index](../docs/README.md)

## Build and launch on an emulator

Build the debug APK with the checked-in Gradle wrapper. With the local Kind services running, forward the API ports to the emulator. The `kind-up.sh` script starts the host port-forwards; apply `adb reverse` after the emulator is attached:

```sh
./gradlew assembleDebug
adb reverse tcp:18081 tcp:18081
adb reverse tcp:18082 tcp:18082
adb reverse tcp:19000 tcp:19000
adb install app/build/outputs/apk/debug/app-debug.apk
adb shell am start -n br.com.opentube.mobile/.MainActivity
```

The phone client also needs the identity, video, and MinIO reverse ports; TV playback needs the video and MinIO ports. Use synthetic demo media and the seeded local creator account only.


## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
