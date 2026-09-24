# android-tv-frontend

Native Kotlin/Compose for TV client for Android TV. It supports public browse/search/infinite feed and HLS playback with D-pad navigation, visible focus, and focus restoration when another page is appended. Upload is not available on TV.

> Documentation: [project index](../docs/README.md) · [repository overview](../README.md)

Use the checked-in Gradle wrapper: `./gradlew testDebugUnitTest` and `./gradlew connectedDebugAndroidTest`. The TV emulator uses the local API and MinIO endpoints exposed with `adb reverse`.

## Component ownership, prerequisites, and lifecycle

- **Owner:** `opentube` / `android-tv-frontend`.
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
adb reverse tcp:18082 tcp:18082
adb reverse tcp:19000 tcp:19000
adb install app/build/outputs/apk/debug/app-debug.apk
adb shell am start -n br.com.opentube.tv/.MainActivity
```

The phone client also needs the identity, video, and MinIO reverse ports; TV playback needs the video and MinIO ports. Use synthetic demo media and the seeded local creator account only.


## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
