use super::*;
use crate::fs::hostfs::IntoFsError;
use std::untrusted::fs;

pub struct StatINode;

impl StatINode {
    pub fn new() -> Arc<dyn INode> {
        Arc::new(File::new(Self))
    }
}

impl ProcINode for StatINode {
    fn generate_data_in_bytes(&self) -> vfs::Result<Vec<u8>> {
        Ok(fs::read_to_string("/proc/stat")
            .map_err(|e| e.into_fs_error())?
            .into_bytes())
    }
}
