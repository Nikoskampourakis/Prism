use quick_xml::de::from_str;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigPreview {
    pub name: String,
    pub android_version: String,
    pub ram_mib: u32,
    pub storage_gib: u32,
    pub operations: Vec<PlannedOperation>,
    pub unsupported: Vec<String>,
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlannedOperation {
    ConfigureDisplay {
        width: Option<u32>,
        height: Option<u32>,
        orientation: Option<String>,
        fullscreen: Option<bool>,
    },
    InstallApk {
        source: String,
    },
    RemovePackage {
        package: String,
    },
    ConfigureRoot {
        enabled: bool,
    },
    ConfigureShizuku {
        enabled: bool,
    },
    ConfigureSharing {
        enabled: bool,
        directory: String,
    },
    GuestAdbCommand {
        command: String,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("invalid XML configuration: {0}")]
    Xml(String),
    #[error("unsupported configuration version {0}; expected version 1")]
    Version(u8),
    #[error("invalid configuration: {0}")]
    Invalid(String),
}

#[derive(Debug, Deserialize)]
#[serde(rename = "prism-config", deny_unknown_fields)]
struct RawConfig {
    #[serde(rename = "@version")]
    version: u8,
    vm: RawVm,
    #[serde(default)]
    display: RawDisplay,
    #[serde(default)]
    applications: RawApplications,
    #[serde(default)]
    system: RawSystem,
    #[serde(default)]
    sharing: RawSharing,
    #[serde(default)]
    adb: RawAdb,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawVm {
    name: String,
    #[serde(rename = "android-version")]
    android_version: String,
    ram: u32,
    storage: u32,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawDisplay {
    resolution: Option<RawResolution>,
    orientation: Option<String>,
    fullscreen: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawResolution {
    #[serde(rename = "@width")]
    width: u32,
    #[serde(rename = "@height")]
    height: u32,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawApplications {
    install: Vec<RawInstall>,
    remove: Vec<RawRemove>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawInstall {
    #[serde(rename = "@source")]
    source: String,
    #[serde(rename = "@path")]
    path: Option<String>,
    #[serde(rename = "@url")]
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRemove {
    #[serde(rename = "@package")]
    package: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawSystem {
    root: Option<RawEnabled>,
    shizuku: Option<RawEnabled>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEnabled {
    #[serde(rename = "@enabled")]
    enabled: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawSharing {
    enabled: Option<bool>,
    #[serde(rename = "shared-directory")]
    shared_directory: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawAdb {
    command: Vec<String>,
}

pub fn parse_and_validate(xml: &str) -> Result<ConfigPreview, ConfigError> {
    let raw: RawConfig = from_str(xml).map_err(|error| ConfigError::Xml(error.to_string()))?;
    if raw.version != 1 {
        return Err(ConfigError::Version(raw.version));
    }

    let name = raw.vm.name.trim();
    if name.is_empty() || name.len() > 80 {
        return Err(ConfigError::Invalid(
            "VM name must be 1 to 80 characters".into(),
        ));
    }
    if raw.vm.android_version.trim().is_empty() {
        return Err(ConfigError::Invalid("Android version is required".into()));
    }
    if !(256..=65_536).contains(&raw.vm.ram) {
        return Err(ConfigError::Invalid(
            "RAM must be between 256 and 65536 MiB".into(),
        ));
    }
    if !(1..=4096).contains(&raw.vm.storage) {
        return Err(ConfigError::Invalid(
            "storage must be between 1 and 4096 GiB".into(),
        ));
    }
    if let Some(resolution) = &raw.display.resolution {
        if !(240..=7680).contains(&resolution.width) || !(320..=7680).contains(&resolution.height) {
            return Err(ConfigError::Invalid(
                "display resolution is outside supported bounds".into(),
            ));
        }
    }
    if let Some(orientation) = &raw.display.orientation {
        if orientation != "portrait" && orientation != "landscape" {
            return Err(ConfigError::Invalid(
                "orientation must be portrait or landscape".into(),
            ));
        }
    }

    let mut operations = Vec::new();
    let mut unsupported = Vec::new();
    if raw.display.resolution.is_some()
        || raw.display.orientation.is_some()
        || raw.display.fullscreen.is_some()
    {
        let resolution = raw.display.resolution;
        operations.push(PlannedOperation::ConfigureDisplay {
            width: resolution.as_ref().map(|value| value.width),
            height: resolution.map(|value| value.height),
            orientation: raw.display.orientation,
            fullscreen: raw.display.fullscreen,
        });
        unsupported.push("guest display configuration requires an active backend".into());
    }
    for install in raw.applications.install {
        let source = match install.source.as_str() {
            "local" => {
                let path = install.path.ok_or_else(|| {
                    ConfigError::Invalid("local APK install requires a path".into())
                })?;
                if !is_safe_relative_path(&path) {
                    return Err(ConfigError::Invalid(
                        "APK paths must be relative and cannot traverse directories".into(),
                    ));
                }
                format!("local:{path}")
            }
            "url" => {
                let url = install
                    .url
                    .ok_or_else(|| ConfigError::Invalid("URL APK install requires a URL".into()))?;
                let parsed = url::Url::parse(&url).map_err(|_| {
                    ConfigError::Invalid("APK URL must be a valid HTTPS URL".into())
                })?;
                if parsed.scheme() != "https"
                    || parsed.host_str().is_none()
                    || !parsed.username().is_empty()
                    || parsed.password().is_some()
                {
                    return Err(ConfigError::Invalid("APK URLs must use HTTPS".into()));
                }
                format!("url:{url}")
            }
            _ => {
                return Err(ConfigError::Invalid(
                    "APK source must be local or url".into(),
                ));
            }
        };
        operations.push(PlannedOperation::InstallApk { source });
        unsupported.push("APK installation requires an active guest backend".into());
    }
    for remove in raw.applications.remove {
        if !is_package_name(&remove.package) {
            return Err(ConfigError::Invalid(format!(
                "invalid Android package name: {}",
                remove.package
            )));
        }
        operations.push(PlannedOperation::RemovePackage {
            package: remove.package,
        });
        unsupported.push("guest package removal requires an active guest backend".into());
    }
    for (name, enabled) in [
        ("root", raw.system.root.map(|item| item.enabled)),
        ("Shizuku", raw.system.shizuku.map(|item| item.enabled)),
    ] {
        if let Some(enabled) = enabled {
            if enabled {
                unsupported.push(format!(
                    "{name} configuration is not available without a compatible guest backend"
                ));
            }
            let operation = if name == "root" {
                PlannedOperation::ConfigureRoot { enabled }
            } else {
                PlannedOperation::ConfigureShizuku { enabled }
            };
            operations.push(operation);
        }
    }

    let sharing_enabled = raw.sharing.enabled.unwrap_or(false);
    let has_shared_directory = raw.sharing.shared_directory.is_some();
    let shared_directory = raw
        .sharing
        .shared_directory
        .unwrap_or_else(|| "PrismShared".into());
    if shared_directory.trim().is_empty()
        || shared_directory.contains(['/', '\\'])
        || shared_directory == "."
        || shared_directory == ".."
    {
        return Err(ConfigError::Invalid(
            "shared directory must be a single folder name".into(),
        ));
    }
    if sharing_enabled {
        unsupported.push("shared-folder access requires a platform sharing integration".into());
    }
    if raw.sharing.enabled.is_some() || has_shared_directory {
        operations.push(PlannedOperation::ConfigureSharing {
            enabled: sharing_enabled,
            directory: shared_directory,
        });
    }

    for command in raw.adb.command {
        if !is_approved_guest_command(&command) {
            return Err(ConfigError::Invalid(format!(
                "ADB command is not in Prism's guest command allowlist: {command}"
            )));
        }
        operations.push(PlannedOperation::GuestAdbCommand { command });
        unsupported.push(
            "guest ADB execution requires explicit permission and an active guest backend".into(),
        );
    }

    let requires_confirmation = operations.iter().any(|operation| {
        matches!(operation, PlannedOperation::RemovePackage { .. })
            || matches!(operation, PlannedOperation::ConfigureRoot { enabled: true })
            || matches!(operation, PlannedOperation::GuestAdbCommand { command } if command.starts_with("settings put ") || command.starts_with("pm uninstall "))
    });

    Ok(ConfigPreview {
        name: name.into(),
        android_version: raw.vm.android_version,
        ram_mib: raw.vm.ram,
        storage_gib: raw.vm.storage,
        operations,
        unsupported,
        requires_confirmation,
    })
}

fn is_safe_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

fn is_package_name(package: &str) -> bool {
    let mut segments = package.split('.');
    segments.all(|segment| {
        !segment.is_empty()
            && segment
                .chars()
                .next()
                .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
            && segment
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
    }) && package.contains('.')
}

fn is_approved_guest_command(command: &str) -> bool {
    let words: Vec<_> = command.split_whitespace().collect();
    match words.as_slice() {
        ["pm", "list", "packages"] => true,
        ["pm", "uninstall", package] => is_package_name(package),
        ["settings", "get", namespace, key] | ["settings", "put", namespace, key, _]
            if matches!(*namespace, "system" | "secure" | "global") =>
        {
            !key.is_empty()
        }
        ["wm", "size", dimensions] => dimensions.split_once('x').is_some_and(|(width, height)| {
            width
                .parse::<u32>()
                .is_ok_and(|value| (240..=7680).contains(&value))
                && height
                    .parse::<u32>()
                    .is_ok_and(|value| (320..=7680).contains(&value))
        }),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"<prism-config version="1"><vm><name>Test VM</name><android-version>13</android-version><ram>4096</ram><storage>32</storage></vm><display><resolution width="1080" height="2400"/><orientation>portrait</orientation><fullscreen>true</fullscreen></display><applications><install source="local" path="apps/example.apk"/><remove package="example.unwanted"/></applications><system><root enabled="false"/><shizuku enabled="false"/></system><sharing><enabled>true</enabled><shared-directory>PrismShared</shared-directory></sharing><adb><command>pm list packages</command></adb></prism-config>"#;

    #[test]
    fn validates_and_previews_operations_without_executing_them() {
        let preview = parse_and_validate(VALID).unwrap();
        assert_eq!(preview.name, "Test VM");
        assert_eq!(preview.operations.len(), 7);
        assert!(matches!(
            preview.operations[0],
            PlannedOperation::ConfigureDisplay {
                width: Some(1080),
                height: Some(2400),
                ..
            }
        ));
        assert!(preview.requires_confirmation);
        assert!(!preview.unsupported.is_empty());
    }

    #[test]
    fn rejects_unknown_xml_elements() {
        let xml = VALID.replace("</vm>", "<host-command>whoami</host-command></vm>");
        assert!(matches!(parse_and_validate(&xml), Err(ConfigError::Xml(_))));
    }

    #[test]
    fn rejects_path_traversal_and_non_https_downloads() {
        let path = VALID.replace("apps/example.apk", "../../secret.apk");
        assert!(matches!(
            parse_and_validate(&path),
            Err(ConfigError::Invalid(_))
        ));
        let url = VALID.replace(
            "source=\"local\" path=\"apps/example.apk\"",
            "source=\"url\" url=\"http://example.test/a.apk\"",
        );
        assert!(matches!(
            parse_and_validate(&url),
            Err(ConfigError::Invalid(_))
        ));
    }

    #[test]
    fn only_approved_guest_commands_are_accepted() {
        let command = VALID.replace("pm list packages", "sh -c 'id'");
        assert!(matches!(
            parse_and_validate(&command),
            Err(ConfigError::Invalid(_))
        ));
    }

    #[test]
    fn rejects_unsupported_schema_versions() {
        let xml = VALID.replace("version=\"1\"", "version=\"2\"");
        assert_eq!(parse_and_validate(&xml), Err(ConfigError::Version(2)));
    }
}
