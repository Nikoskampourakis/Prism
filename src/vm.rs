use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmRecord {
    pub id: Uuid,
    pub name: String,
    pub android_version: String,
    pub status: VmStatus,
    pub last_launched_unix: Option<u64>,
    pub allocated_ram_mib: u32,
    pub allocated_storage_gib: u32,
    pub used_storage_bytes: Option<u64>,
    pub backend_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VmStatus {
    Stopped,
    Running,
    Suspended,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BackendCapabilities {
    pub create: bool,
    pub start: bool,
    pub stop: bool,
    pub suspend: bool,
    pub resume: bool,
    pub snapshot: bool,
    pub restore: bool,
    pub import: bool,
    pub export: bool,
    pub resource_monitoring: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BackendError {
    #[error("operation is unsupported by this backend: {0}")]
    Unsupported(&'static str),
    #[error("backend operation failed: {0}")]
    Failed(String),
}

pub trait VmBackend {
    fn capabilities(&self) -> BackendCapabilities;
    fn create(&self, name: &str) -> Result<VmRecord, BackendError>;
    fn start(&self, id: Uuid) -> Result<(), BackendError>;
    fn stop(&self, id: Uuid) -> Result<(), BackendError>;
    fn suspend(&self, id: Uuid) -> Result<(), BackendError>;
    fn resume(&self, id: Uuid) -> Result<(), BackendError>;
    fn snapshot(&self, id: Uuid, name: &str) -> Result<(), BackendError>;
    fn restore(&self, id: Uuid, snapshot: &str) -> Result<(), BackendError>;
    fn import(&self, path: &str) -> Result<VmRecord, BackendError>;
    fn export(&self, id: Uuid, path: &str) -> Result<(), BackendError>;
    fn resource_usage(&self, id: Uuid) -> Result<ResourceUsage, BackendError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceUsage {
    pub cpu_percent: Option<u8>,
    pub memory_bytes: Option<u64>,
    pub disk_bytes: Option<u64>,
}

#[derive(Debug, Default)]
pub struct UnavailableBackend;

impl VmBackend for UnavailableBackend {
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::default()
    }
    fn create(&self, _: &str) -> Result<VmRecord, BackendError> {
        Err(BackendError::Unsupported("create"))
    }
    fn start(&self, _: Uuid) -> Result<(), BackendError> {
        Err(BackendError::Unsupported("start"))
    }
    fn stop(&self, _: Uuid) -> Result<(), BackendError> {
        Err(BackendError::Unsupported("stop"))
    }
    fn suspend(&self, _: Uuid) -> Result<(), BackendError> {
        Err(BackendError::Unsupported("suspend"))
    }
    fn resume(&self, _: Uuid) -> Result<(), BackendError> {
        Err(BackendError::Unsupported("resume"))
    }
    fn snapshot(&self, _: Uuid, _: &str) -> Result<(), BackendError> {
        Err(BackendError::Unsupported("snapshot"))
    }
    fn restore(&self, _: Uuid, _: &str) -> Result<(), BackendError> {
        Err(BackendError::Unsupported("restore"))
    }
    fn import(&self, _: &str) -> Result<VmRecord, BackendError> {
        Err(BackendError::Unsupported("import"))
    }
    fn export(&self, _: Uuid, _: &str) -> Result<(), BackendError> {
        Err(BackendError::Unsupported("export"))
    }
    fn resource_usage(&self, _: Uuid) -> Result<ResourceUsage, BackendError> {
        Err(BackendError::Unsupported("resource monitoring"))
    }
}

pub struct VmManager<B> {
    backend: B,
}

impl<B: VmBackend> VmManager<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }
    pub fn capabilities(&self) -> BackendCapabilities {
        self.backend.capabilities()
    }

    pub fn create(&self, name: &str) -> Result<VmRecord, BackendError> {
        self.backend.create(name)
    }
    pub fn suspend(&self, id: Uuid) -> Result<(), BackendError> {
        self.backend.suspend(id)
    }
    pub fn resume(&self, id: Uuid) -> Result<(), BackendError> {
        self.backend.resume(id)
    }
    pub fn snapshot(&self, id: Uuid, name: &str) -> Result<(), BackendError> {
        self.backend.snapshot(id, name)
    }
    pub fn restore(&self, id: Uuid, snapshot: &str) -> Result<(), BackendError> {
        self.backend.restore(id, snapshot)
    }
    pub fn import(&self, path: &str) -> Result<VmRecord, BackendError> {
        self.backend.import(path)
    }
    pub fn export(&self, id: Uuid, path: &str) -> Result<(), BackendError> {
        self.backend.export(id, path)
    }
    pub fn resource_usage(&self, id: Uuid) -> Result<ResourceUsage, BackendError> {
        self.backend.resource_usage(id)
    }

    pub fn start(&self, vm: &mut VmRecord) -> Result<(), BackendError> {
        self.backend.start(vm.id)?;
        vm.status = VmStatus::Running;
        vm.last_launched_unix = Some(unix_timestamp());
        Ok(())
    }

    pub fn stop(&self, vm: &mut VmRecord) -> Result<(), BackendError> {
        self.backend.stop(vm.id)?;
        vm.status = VmStatus::Stopped;
        Ok(())
    }
}

fn unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_backend_never_marks_a_vm_running() {
        let manager = VmManager::new(UnavailableBackend);
        let mut vm = VmRecord {
            id: Uuid::new_v4(),
            name: "No engine".into(),
            android_version: "13".into(),
            status: VmStatus::Stopped,
            last_launched_unix: None,
            allocated_ram_mib: 2048,
            allocated_storage_gib: 16,
            used_storage_bytes: None,
            backend_id: None,
        };
        assert_eq!(
            manager.start(&mut vm),
            Err(BackendError::Unsupported("start"))
        );
        assert_eq!(vm.status, VmStatus::Stopped);
        assert_eq!(vm.last_launched_unix, None);
        assert_eq!(manager.capabilities(), BackendCapabilities::default());
    }
}
