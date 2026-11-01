# XML Configuration, Version 1

Profiles are parsed into a preview. Parsing does not create a VM, fetch an APK, invoke ADB, alter the host, or change guest settings. A future executor must check backend capabilities, obtain relevant host/guest permissions, require confirmation for destructive actions, report each operation, and roll back only operations it can safely reverse.

Unknown elements and attributes are rejected. The root must be `prism-config` with `version="1"`. Numeric limits are checked before a preview is returned.

## Example

```xml
<prism-config version="1">
  <vm>
    <name>My Android VM</name>
    <android-version>13</android-version>
    <ram>4096</ram>
    <storage>32</storage>
  </vm>
  <display>
    <resolution width="1080" height="2400"/>
    <orientation>portrait</orientation>
    <fullscreen>true</fullscreen>
  </display>
  <applications>
    <install source="local" path="apps/example.apk"/>
    <install source="url" url="https://downloads.example.invalid/example.apk"/>
    <remove package="example.unwanted"/>
  </applications>
  <system>
    <root enabled="false"/>
    <shizuku enabled="false"/>
  </system>
  <sharing>
    <enabled>true</enabled>
    <shared-directory>PrismShared</shared-directory>
  </sharing>
  <adb>
    <command>pm list packages</command>
  </adb>
</prism-config>
```

The `.invalid` domain is reserved for examples. A real download URL must be supplied by the user and use HTTPS. No downloads are started by parsing this profile.

## Accepted fields

- `vm`: required `name`, `android-version`, `ram` in MiB (256-65536), and `storage` in GiB (1-4096).
- `display`: optional resolution (width 240-7680, height 320-7680), `portrait` or `landscape` orientation, and fullscreen boolean. These values appear in the preview; display control is unsupported without a backend.
- `applications/install`: repeatable. `source="local"` requires a relative `path` without parent traversal. `source="url"` requires a credential-free HTTPS URL with a host. Install is always a guest operation and is not executed today.
- `applications/remove`: repeatable Android package names. Destructive and requires confirmation in any future executor.
- `system/root` and `system/shizuku`: optional `enabled` attributes. This records guest requests only; neither implies host root or a functioning Shizuku integration. Enabling guest root is confirmation-gated and unsupported today.
- `sharing`: optional enabled flag and a single-folder `shared-directory` name. Sharing is unsupported today.
- `adb/command`: optional repeatable guest commands from the narrow allowlist below. There is no arbitrary command or host shell operation.

## Guest command allowlist

The current parser accepts only:

- `pm list packages`
- `pm uninstall <package.name>`
- `settings get system|secure|global <key>`
- `settings put system|secure|global <key> <single-token-value>`
- `wm size <width>x<height>` within the display bounds

Commands are parsed as whitespace-separated tokens; quoted values with spaces are not supported. Package removal and settings writes require confirmation. Every ADB operation is marked unsupported until a guest ADB transport, explicit permission, and backend exist. The parser never calls a process API.

## Preview and errors

`parse_and_validate` returns VM sizing plus a list of typed `PlannedOperation`s, unsupported-operation explanations, and `requires_confirmation`. It rejects invalid XML/schema, unsupported versions, unsafe paths, unknown attributes/elements, out-of-range values, non-HTTPS URLs, malformed package names, and non-allowlisted commands. Callers must display the entire preview and cannot treat an accepted parse as authorization to execute.

Per-operation results, profile import/export UI, reusable profile storage, execution, and rollback are planned, not implemented.