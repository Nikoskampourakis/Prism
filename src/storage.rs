use crate::vm::VmRecord;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("storage I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("metadata could not be decoded: {0}")]
    Json(#[from] serde_json::Error),
    #[error("VM catalog contains duplicate IDs")]
    DuplicateId,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Catalog {
    #[serde(default)]
    vms: Vec<VmRecord>,
}

#[derive(Debug, Clone)]
pub struct VmCatalog {
    path: PathBuf,
}

impl VmCatalog {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn load(&self) -> Result<Vec<VmRecord>, StorageError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let catalog: Catalog = serde_json::from_slice(&fs::read(&self.path)?)?;
        validate_unique_ids(&catalog.vms)?;
        Ok(catalog.vms)
    }

    pub fn save(&self, vms: &[VmRecord]) -> Result<(), StorageError> {
        validate_unique_ids(vms)?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(&Catalog { vms: vms.to_vec() })?;
        let temporary = self.path.with_extension("json.tmp");
        fs::write(&temporary, bytes)?;
        replace_file(&temporary, &self.path)?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

fn validate_unique_ids(vms: &[VmRecord]) -> Result<(), StorageError> {
    let mut ids = std::collections::HashSet::with_capacity(vms.len());
    if vms.iter().any(|vm| !ids.insert(vm.id)) {
        return Err(StorageError::DuplicateId);
    }
    Ok(())
}

fn replace_file(temporary: &Path, destination: &Path) -> Result<(), std::io::Error> {
    #[cfg(windows)]
    if destination.exists() {
        fs::remove_file(destination)?;
    }
    fs::rename(temporary, destination)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::VmStatus;
    use uuid::Uuid;

    fn vm() -> VmRecord {
        VmRecord {
            id: Uuid::new_v4(),
            name: "Test VM".into(),
            android_version: "13".into(),
            status: VmStatus::Stopped,
            last_launched_unix: None,
            allocated_ram_mib: 2048,
            allocated_storage_gib: 32,
            used_storage_bytes: None,
            backend_id: None,
        }
    }

    #[test]
    fn saves_and_reloads_vm_metadata() {
        let directory = tempfile::tempdir().unwrap();
        let catalog = VmCatalog::new(directory.path().join("state/vms.json"));
        let expected = vec![vm()];
        catalog.save(&expected).unwrap();
        assert_eq!(catalog.load().unwrap(), expected);
    }

    #[test]
    fn rejects_duplicate_ids() {
        let directory = tempfile::tempdir().unwrap();
        let catalog = VmCatalog::new(directory.path().join("vms.json"));
        let first = vm();
        let mut duplicate = vm();
        duplicate.id = first.id;
        assert!(matches!(
            catalog.save(&[first, duplicate]),
            Err(StorageError::DuplicateId)
        ));
    }
}
