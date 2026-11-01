# Prism

Prism is a free, open-source project for managing virtual Android devices. Its shared application core and UI are written in Rust and Slint. The project is designed to grow across platforms without maintaining separate Android and iOS application implementations.

**Current status: early foundation. Prism does not currently boot Android or any other guest operating system.** There is no VM backend, ROM download source, APK installer, or host-to-guest file transport yet. The UI reports this explicitly; it never marks a guest as running unless a backend confirms success.

## Features and Status

| Area | Status | Notes |
| --- | --- | --- |
| Slint desktop dashboard | Implemented | Navigation, persistent VM count, and explicit unavailable-engine state. |
| Shared Rust VM manager | Implemented | Backend contract and lifecycle safety; only the unavailable backend is included. |
| VM metadata | Implemented | JSON catalog with duplicate-ID checks and staged writes. No guest disks are created. |
| XML configuration | Implemented | Strict version-1 parsing, validation, operation preview, unsupported-operation reporting, and confirmation flags. No operations execute. |
| ROM compatibility model | Implemented | Evaluates architecture, format, system image, kernel/boot image, and firmware against a backend capability description. It does not inspect image binaries yet. |
| APK intake checks | Implemented | Local ZIP/APK signature and extension checks; URL syntax validation. Package identity extraction and installation are not implemented. |
| Accessibility preferences | Implemented | Shared Rust preferences and text-scale bounds. Native screen-reader integration is not implemented. |
| Android virtualization | Unsupported today | No backend has a supported, ordinary-app integration in this project. Device and OS capabilities vary. |
| VM lifecycle, fullscreen guest, snapshots, compression | Planned | Requires a real backend and platform integration. |
| ROM discovery/downloads, APK queues, file sharing | Planned | No download source or guest transport is configured. |
| iOS application and Android guest support | Unsupported today | Shared-core direction only; no iOS target or guest virtualization implementation. |

Experimental means an integration has been prototyped but is not reliable. Prism currently has no experimental VM backend.

## Screenshots

No screenshots are available yet. The application window is an early dashboard shell; it intentionally contains no fictional VM cards or running guest preview.

## Technology

- Rust 2024 and Cargo
- Slint 1.x for shared UI
- `quick-xml` and Serde for XML profile parsing
- JSON for the small local VM metadata catalog
- GPL-3.0-only

## Architecture

The Slint UI is a thin front end over shared Rust modules. `VmBackend` defines lifecycle, snapshot, import/export, and resource-monitoring operations. A real backend must report its capabilities; the included `UnavailableBackend` rejects every operation. The ROM compatibility model uses backend-reported architectures, image formats, and boot requirements instead of assuming a downloaded image is bootable.

See [architecture](docs/architecture.md), [backend design](docs/vm-backends.md), and [platform support](docs/platform-support.md).

## Installation and Development

### Requirements

- Rust stable with Cargo (Rust 1.92 or newer)
- A native desktop toolchain supported by Slint
- On Linux, a C compiler and the windowing development libraries required by Slint's Winit backend (commonly X11, Wayland, xkbcommon, and fontconfig headers)

Install Rust with [rustup](https://rustup.rs/). On Debian or Ubuntu, the following package set is a useful starting point:

```sh
sudo apt install build-essential pkg-config libx11-dev libxcursor-dev libxi-dev \
	libxkbcommon-dev libwayland-dev libfontconfig1-dev
```

### Run the desktop shell

```sh
git clone https://github.com/prism-vm/prism.git
cd prism
cargo run
```

The repository URL is a placeholder until the project has a canonical upstream. If working from a local checkout, omit the clone step.

### Test and build

```sh
cargo test
cargo build --release
```

`cargo test` covers XML validation, VM state safety, metadata persistence, ROM compatibility decisions, APK intake, and accessibility preference validation. Android and iOS packaging are not currently build targets in CI or this repository.

## Supported Platforms

| Platform | UI/core | VM execution |
| --- | --- | --- |
| Linux desktop | Buildable and tested | Unsupported; no backend |
| Windows/macOS desktop | Slint is cross-platform; not verified by this project yet | Unsupported; no backend |
| Android | Primary design target; app packaging and native integration are not implemented | Unsupported; ordinary-app VM backend unavailable here |
| iOS | Shared-core direction only; no target configured | Unsupported; no Android guest, arbitrary guest OS, or guest app execution |

Android's Virtualization Framework is an OS/device facility, not a capability Prism can assume is exposed to ordinary third-party apps. See [platform support](docs/platform-support.md) before proposing a backend. Running a userspace emulator such as QEMU would require separate engineering, licensing, performance, and device-compatibility evaluation; it is not included.

## VM Backend Status

The backend contract covers create, start, stop, suspend, resume, snapshot, restore, import, export, and resource usage. Operations fail with an explicit unsupported error when no backend exists. VM records are metadata only; they do not imply that disk images or guest processes exist.

## XML Configuration

Version 1 profiles are parsed and validated, then returned as a preview. The parser rejects unknown elements, unsafe relative paths, unsupported versions, malformed URLs, and guest commands outside Prism's allowlist. It never starts host processes. Guest actions remain unexecuted until a permission-checked backend exists. Destructive actions are marked for confirmation.

See the [XML guide](docs/xml-configuration.md) for the documented schema, operation statuses, and an example profile.

## APK Management

The shared core can check that a selected local file has an `.apk` suffix and ZIP signature, and can validate an explicitly supplied HTTPS URL. Local APKs remain in `AwaitingUserConfirmation`; package name extraction, download, guest installation, removal, queues, and history are not implemented. A ZIP signature alone does not establish that a file is a valid or safe Android package.

## ROM Library

The ROM compatibility model produces Compatible, Compatible with limitations, Experimental, or Unsupported results based on the selected backend's declared capabilities and inspected metadata. Current code does not inspect image contents, download ROMs, verify checksums, import disks, or boot guests. There are no configured ROM publishers or community repositories.

## File Sharing

No host directory is exposed to a guest. Per-VM shared folders, transfer progress, conflict handling, clipboard integration, and downloads are planned behind a permission-scoped platform interface.

## Known Limitations

- This is a maintainable starting point, not yet a usable VM launcher.
- The desktop shell is the only verified UI target; Android and iOS packaging needs native build integration.
- Current metadata persistence stores VM records in JSON; guest disk and profile catalog lifecycle are not implemented.
- The XML parser creates a preview only. It does not apply profiles or perform rollback.
- Accessibility preferences are Rust-side data; native accessibility announcements and guest accessibility are separate future work.
- Compression, root, Shizuku, ADB transport, downloads, and guest fullscreen are not available.
- VMOS is used only as a public functional reference; see [research notes](docs/research-vmos.md). Prism does not use its source, assets, or proprietary implementation.

## Roadmap

1. Define and test backend capability negotiation on a target device that exposes a supported virtualization API.
2. Add safe profile selection and preview UI, including per-operation results and rollback semantics.
3. Implement scoped host storage and an image inspection/import wizard.
4. Add opt-in APK acquisition and package metadata extraction with explicit user confirmation.
5. Add VM lifecycle, display/input, resource monitoring, and fullscreen only after a real backend is validated.
6. Add compression, sharing, and native accessibility adapters behind shared Rust interfaces.
7. Establish Android packaging and device test coverage; evaluate iOS separately without promising Android guests.

## Contributing

Contributions should preserve the explicit capability boundary: never report a VM running because a request was sent, and never interpret guest commands as host shell commands. Start with [CONTRIBUTING.md](CONTRIBUTING.md), run `cargo fmt --check` and `cargo test`, and describe platform/device coverage for backend changes.

## License

Prism is licensed under [GNU GPL v3.0 only](LICENSE).
