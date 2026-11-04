//! Not built yet. The tests in `tests/lab_definitions.rs` come first.
use super::pack::PackContent;
use crate::lab::Result;
use std::collections::BTreeMap;
use std::path::Path;

pub const RRFLOW_PRODUCT: &str = "rrflow";

pub struct RrflowContent;

impl PackContent for RrflowContent {
    fn product(&self) -> &str {
        RRFLOW_PRODUCT
    }

    fn schema(&self) -> u16 {
        1
    }

    fn check(&self, _files: &BTreeMap<String, String>) -> Result<()> {
        Err("not built yet".into())
    }

    fn notable_changes(
        &self,
        _before: &BTreeMap<String, String>,
        _after: &BTreeMap<String, String>,
    ) -> Result<Vec<String>> {
        Err("not built yet".into())
    }
}

pub fn lab_files(_lab: &Path) -> Result<BTreeMap<String, String>> {
    Err("not built yet".into())
}
