# VMOS Functional Reference Research

**Research date:** 2026-11-01

## Scope and limitations

VMOS is treated as a functional reference, not a source of code or assets. This research used its publicly accessible website and publicly listed product information only; Prism did not inspect private source, bypass product controls, or reuse VMOS visual assets. The current `vmos.com` homepage is branded as a product matrix for cloud-device management, multi-account operations, and customer-service tools. It does not, by itself, establish the current behavior of a local VMOS Android app. Public listings and product versions can change by region and date.

Because no VMOS app was installed or behaviorally tested in this work, statements about local VM lifecycle and guest capabilities below are framed as workflow categories to study, not verified claims about a current release.

## Functional areas relevant to Prism

| Area to study | Independent Prism direction | Status |
| --- | --- | --- |
| VM list and per-instance management | One shared dashboard and backend-neutral VM record | Dashboard foundation implemented; lifecycle unavailable |
| Guest launch/return and status | Backend-confirmed status and state reconciliation | Contract implemented; backend unavailable |
| Virtual device presentation | Phone-shaped guest display with orientation, input, and overlay controls | Planned; no guest display surface |
| Per-VM configuration | Versioned XML profile with preview and permission checks | Parser and preview model implemented; execution unavailable |
| App installation/removal | Explicit APK source, package inspection, confirmation, and guest install result | Intake checks implemented; install unavailable |
| Host/guest files | Explicit per-VM shared-directory capability and transfer policy | Planned; no filesystem exposure |
| Root/advanced controls | Separate host permission, guest root, guest ADB, and Shizuku capabilities | Parsed as requested settings; no integration |
| Storage/compression | Backend-owned disk operation, staged output, verification, and restore | Planned; no disk backend |

## Design independence

Prism uses its own Rust domain models, XML schema, capability-negotiated backend interface, and Slint layout. No VMOS source code, images, icons, strings, or proprietary implementation details were copied. The dashboard's honest empty state and unsupported-operation labels are Prism-specific choices, not a recreation of a VMOS screen.

## Sources

- [VMOS public website](https://vmos.com/) (currently promotes cloud-device products; not evidence for local VM internals)
- [VMOS Pro public app listing](https://play.google.com/store/apps/details?id=com.vmos.pro) (availability and listing content vary over time and by region)

Research should be revisited against current public documentation and behavior before making product-compatibility claims. Any future comparison must remain observational and independently implemented.