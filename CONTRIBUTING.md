# Contributing to Prism

Prism is early-stage software. Contributions that improve the shared Rust foundation, tests, build reproducibility, or honest platform capability reporting are especially useful.

## Development checks

```sh
cargo fmt --check
cargo test
cargo build --release
```

Use the stable Rust toolchain. Keep platform-specific code behind explicit interfaces and avoid introducing a second application implementation. Do not add analytics, background services, or external download sources without a clear user-controlled need.

## Backend changes

- Advertise only capabilities that the backend can actually perform on the current device.
- Update VM state only after the backend confirms success.
- Document device, OS, privilege, and image prerequisites.
- Never execute profile-provided commands on the host. Guest commands need a guest transport, a narrow allowlist, and explicit permission.
- Test unsupported and failure paths, not just success paths.

## UI and documentation

Label incomplete behavior as planned, experimental, or unsupported. Do not add sample VMs that appear live. Include accessibility labels and keyboard behavior in new controls. Update the platform and XML documentation when their contracts change.

By contributing, you agree that your contributions are offered under the project's GPL-3.0-only license.