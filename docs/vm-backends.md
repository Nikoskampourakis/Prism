# VM Backends

## Contract

`VmBackend` in `src/vm.rs` defines create, start, stop, suspend, resume, snapshot, restore, import, export, and resource-usage operations. `BackendCapabilities` defaults every capability to false. `UnavailableBackend` returns `BackendError::Unsupported` for every operation.

The manager updates a record to Running only after `start` returns success. The backend must be the source of truth for guest state and resource metrics. A submitted operation is not evidence that it succeeded.

## Android

Android's Virtualization Framework and protected KVM are platform/OEM capabilities with device, OS, and permission requirements. Their existence does not imply that a third-party app can use them on arbitrary consumer devices. Prism currently has no verified public ordinary-app integration and no privileged/system-app deployment model. Do not enable a capability based only on Android version or the presence of a device node.

Userspace emulation is a separate possible research direction, not a backend. It would need architecture/performance evaluation, a reviewed guest image supply chain, display and input integration, and license review. No such engine is bundled.

## Backend acceptance checklist

- Identify an accessible and documented API for ordinary apps on supported devices.
- State hardware, Android release, privilege, and image-format constraints.
- Return capabilities per device and per operation.
- Keep guest root separate from host Android permissions, guest ADB, and Shizuku.
- Implement truthful state reconciliation and resource reporting.
- Gate file sharing, APK installation, networking, display/input, suspend, and snapshots independently.
- Test unsupported, permission-denied, cancellation, partial failure, and recovery paths.
- Do not run ROM-supplied scripts automatically.

## Compression and disks

No disk backend or compression implementation exists. Future compression must operate on a stopped/consistent guest, write a separate staged archive, verify it before publication, preserve the original, report progress, and support cancellation. Restore must inspect paths and archive contents before writing. Backend capability negotiation must expose these operations; the UI must not imply storage savings before measurement.