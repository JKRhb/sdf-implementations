use std::sync::atomic::{AtomicU64, Ordering};

#[derive(serde::Serialize, Debug, Clone)]
pub struct SdfModelEntry {
    id: String,
    pub model: serde_json::Map<String, serde_json::Value>,
    pub version: String,
    pub namespace: String,
    pub lineage: Option<String>,
}

impl PartialOrd for SdfModelEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.version.cmp(&other.version))
    }
}

impl PartialEq for SdfModelEntry {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
            && self.namespace == other.namespace
            && self.lineage == other.lineage
    }
}

impl SdfModelEntry {
    pub fn new(
        model: serde_json::Map<String, serde_json::Value>,
        version: String,
        namespace: String,
        lineage: Option<String>,
    ) -> SdfModelEntry {
        SdfModelEntry {
            id: Self::get_next_model_id(),
            model,
            lineage,
            namespace,
            version,
        }
    }

    fn get_next_model_id() -> String {
        MODEL_ID_SEQ.fetch_add(1, Ordering::SeqCst);
        MODEL_ID_SEQ.load(Ordering::SeqCst).to_string()
    }
}

static MODEL_ID_SEQ: AtomicU64 = AtomicU64::new(0);
