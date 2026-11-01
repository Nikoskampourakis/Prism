# Platform Support

## Status matrix

| Target | Shared Rust/Slint intent | Current state |
| --- | --- | --- |
| Linux desktop | Yes | Desktop shell builds and tests. No VM backend. |
| Windows/macOS desktop | Yes | Slint supports these families; Prism has not verified builds or installers. |
| Android | Primary target | No Android package, activity integration, storage picker, or VM backend in this repository yet. |
| iOS | Shared-core target | No iOS app target or guest VM implementation. |

## Android virtualization constraints

Android's Virtualization Framework (AVF) is part of the Android platform and depends on OEM/device support and platform permissions. Protected KVM is not a general-purpose capability that an ordinary app can presume. Prism has not validated a public API accessible to ordinary apps for booting arbitrary Android guests across consumer devices. The app therefore reports the backend unavailable rather than probing privileged interfaces or claiming compatibility from an Android version number.

An Android packaging path can be added independently of the VM backend, but packaging a Slint UI does not itself grant guest virtualization. Any platform bridge should be small, permission-scoped, and called through a Rust interface.

## iOS constraints

iOS does not provide a general public API for third-party applications to boot arbitrary Android virtual machines. App sandboxing, execution policy, memory, storage, and platform virtualization APIs constrain guest operating systems, filesystems, and executable apps. Prism will not advertise Android guest execution, arbitrary VM images, or unrestricted host/guest sharing on iOS without a genuinely supported implementation.

## Accessibility distinction

Prism's own labels, keyboard navigation, contrast, scale, motion, and announcements are host-UI features. They do not configure accessibility services inside Android guests. Native screen-reader integration belongs in platform adapters; the shared Rust preference model already distinguishes Prism UI settings from guest behavior.

## Research sources

- [Android Open Source Project: Virtualization](https://source.android.com/docs/security/features/virtualization)
- [Android Developers: Virtualization](https://source.android.com/docs/core/virtualization)
- [Apple Virtualization framework](https://developer.apple.com/documentation/virtualization)

These sources describe platform components, not a promise that Prism or an ordinary mobile app has access to every capability.