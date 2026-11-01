# Architecture

## Current shape

```text
Slint UI (ui/main.slint)
        |
        v
Rust application shell (src/main.rs)
        |
        +-- VM manager and backend contract (src/vm.rs)
        +-- XML parse, validation, preview (src/config.rs)
        +-- VM metadata catalog (src/storage.rs)
        +-- ROM compatibility model (src/rom.rs)
        +-- APK intake checks (src/packages.rs)
        +-- accessibility preferences (src/accessibility.rs)
```

The UI is shared Slint markup and application/domain logic is Rust. The current desktop executable reads the metadata catalog and presents the dashboard. UI actions do not call a VM engine because none is included.

## Boundaries

- `VmBackend` is the only authority for guest lifecycle and runtime telemetry. Capability declarations are explicit and defaults are false.
- `VmManager` updates `VmRecord` only after a backend operation succeeds.
- `VmCatalog` persists descriptive metadata; it is not a disk manager and never claims a guest exists on disk.
- `config` turns a profile into a preview. A future executor must separately confirm destructive operations and request guest/platform permissions.
- ROM compatibility is evaluated against backend capabilities. A file extension or successful download cannot establish bootability.
- APK inspection returns no package identity unless a real package parser can establish it. File signature checks are intake hints, not trust verification.
- Host integration belongs behind narrow Rust traits. No Java application implementation is present.

## Persistence

The initial catalog is a small JSON file under the operating-system application data directory. Writes are staged through a temporary file and duplicate IDs are rejected. SQLite is intentionally deferred until query, migration, or transactional needs justify adding it.

## Error and state model

Unsupported operations are errors, not successful no-ops. Runtime status is not inferred from user intent. A future backend must reconcile state with the actual guest process after errors, interruption, or application restart.

## Planned evolution

1. Add backend capability negotiation and operation-result types with observable progress/cancellation.
2. Add durable profile and transfer history only when those workflows are implemented.
3. Add scoped platform adapters for storage picker, guest transport, display/input, and accessibility.
4. Add native packaging per supported target while keeping domain logic shared.

No current backend implements create, boot, snapshots, restore, resource telemetry, compression, or virtual disk management.