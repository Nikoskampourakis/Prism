use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApkSource {
    LocalFile(PathBuf),
    ExplicitUrl(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallState {
    AwaitingUserConfirmation,
    Queued,
    Installed,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApkRecord {
    pub file_name: String,
    pub size_bytes: u64,
    pub source: ApkSource,
    pub package_name: Option<String>,
    pub state: InstallState,
}

#[derive(Debug, Error)]
pub enum ApkError {
    #[error("APK file could not be read: {0}")]
    Io(#[from] std::io::Error),
    #[error("file must have an .apk extension")]
    InvalidExtension,
    #[error("file does not have a ZIP/APK signature")]
    InvalidArchive,
    #[error("download URL must be a valid HTTPS URL with a host")]
    InvalidUrl,
}

pub fn inspect_local_apk(path: &Path) -> Result<ApkRecord, ApkError> {
    if !path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("apk"))
    {
        return Err(ApkError::InvalidExtension);
    }
    let mut file = File::open(path)?;
    let size_bytes = file.metadata()?.len();
    let mut signature = [0; 4];
    file.read_exact(&mut signature)?;
    if signature != [b'P', b'K', 3, 4]
        && signature != [b'P', b'K', 5, 6]
        && signature != [b'P', b'K', 7, 8]
    {
        return Err(ApkError::InvalidArchive);
    }
    Ok(ApkRecord {
        file_name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        size_bytes,
        source: ApkSource::LocalFile(path.to_path_buf()),
        package_name: None,
        state: InstallState::AwaitingUserConfirmation,
    })
}

pub fn validate_download_url(url: &str) -> Result<(), ApkError> {
    let parsed = url::Url::parse(url).map_err(|_| ApkError::InvalidUrl)?;
    if parsed.scheme() != "https"
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err(ApkError::InvalidUrl);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn local_apk_inspection_requires_archive_signature_and_never_queues_install_automatically() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("sample.apk");
        fs::write(&path, [b'P', b'K', 3, 4, 0, 0]).unwrap();
        let record = inspect_local_apk(&path).unwrap();
        assert_eq!(record.state, InstallState::AwaitingUserConfirmation);
        assert_eq!(record.package_name, None);

        fs::write(&path, b"not an APK").unwrap();
        assert!(matches!(
            inspect_local_apk(&path),
            Err(ApkError::InvalidArchive)
        ));
    }

    #[test]
    fn download_urls_must_use_https_without_embedded_credentials() {
        assert!(validate_download_url("https://downloads.example/app.apk").is_ok());
        assert!(matches!(
            validate_download_url("http://downloads.example/app.apk"),
            Err(ApkError::InvalidUrl)
        ));
        assert!(matches!(
            validate_download_url("https://user:pass@example.com/app.apk"),
            Err(ApkError::InvalidUrl)
        ));
    }
}
