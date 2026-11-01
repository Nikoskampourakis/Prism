use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Architecture {
    Aarch64,
    Armv7,
    X86_64,
    X86,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageFormat {
    Raw,
    Qcow2,
    AndroidSparse,
    BootImage,
    VmPackage,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RomInspection {
    pub architecture: Architecture,
    pub android_version: Option<String>,
    pub format: ImageFormat,
    pub has_system_partition: bool,
    pub has_boot_image: bool,
    pub has_kernel: bool,
    pub has_firmware: bool,
    pub requirements_uncertain: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RomBackendCapabilities {
    pub architectures: Vec<Architecture>,
    pub formats: Vec<ImageFormat>,
    pub requires_separate_kernel: bool,
    pub requires_firmware: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compatibility {
    Compatible,
    CompatibleWithLimitations,
    Experimental,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityReport {
    pub status: Compatibility,
    pub reasons: Vec<String>,
}

pub fn assess_compatibility(
    rom: &RomInspection,
    backend: &RomBackendCapabilities,
) -> CompatibilityReport {
    let mut reasons = Vec::new();
    if !backend.architectures.contains(&rom.architecture) {
        reasons.push(format!("backend does not support {:?}", rom.architecture));
    }
    if !backend.formats.contains(&rom.format) {
        reasons.push(format!("backend does not support {:?} images", rom.format));
    }
    if !rom.has_system_partition {
        reasons.push("system partition was not detected".into());
    }
    if backend.requires_separate_kernel && !rom.has_kernel && !rom.has_boot_image {
        reasons.push("backend requires a kernel or boot image that was not detected".into());
    }
    if backend.requires_firmware && !rom.has_firmware {
        reasons.push("backend requires firmware that was not detected".into());
    }
    if !reasons.is_empty() {
        return CompatibilityReport {
            status: Compatibility::Unsupported,
            reasons,
        };
    }
    if rom.requirements_uncertain {
        return CompatibilityReport {
            status: Compatibility::Experimental,
            reasons: vec![
                "image metadata is incomplete; bootability has not been established".into(),
            ],
        };
    }
    if !rom.has_boot_image {
        return CompatibilityReport {
            status: Compatibility::CompatibleWithLimitations,
            reasons: vec![
                "no boot image detected; backend-specific boot components may still be required"
                    .into(),
            ],
        };
    }
    CompatibilityReport {
        status: Compatibility::Compatible,
        reasons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn backend() -> RomBackendCapabilities {
        RomBackendCapabilities {
            architectures: vec![Architecture::Aarch64],
            formats: vec![ImageFormat::Raw],
            requires_separate_kernel: true,
            requires_firmware: true,
        }
    }

    fn rom() -> RomInspection {
        RomInspection {
            architecture: Architecture::Aarch64,
            android_version: Some("13".into()),
            format: ImageFormat::Raw,
            has_system_partition: true,
            has_boot_image: true,
            has_kernel: true,
            has_firmware: true,
            requirements_uncertain: false,
        }
    }

    #[test]
    fn requires_backend_and_image_components_for_compatibility() {
        assert_eq!(
            assess_compatibility(&rom(), &backend()).status,
            Compatibility::Compatible
        );
        let mut incomplete = rom();
        incomplete.has_firmware = false;
        assert_eq!(
            assess_compatibility(&incomplete, &backend()).status,
            Compatibility::Unsupported
        );
    }

    #[test]
    fn unknown_boot_requirements_are_experimental_not_compatible() {
        let mut uncertain = rom();
        uncertain.requirements_uncertain = true;
        assert_eq!(
            assess_compatibility(&uncertain, &backend()).status,
            Compatibility::Experimental
        );
    }

    #[test]
    fn image_without_boot_components_is_not_reported_as_bootable() {
        let mut image = rom();
        image.has_boot_image = false;
        image.has_kernel = false;
        image.has_firmware = false;
        let report = assess_compatibility(&image, &backend());
        assert_eq!(report.status, Compatibility::Unsupported);
        assert!(!report.reasons.is_empty());
    }
}
