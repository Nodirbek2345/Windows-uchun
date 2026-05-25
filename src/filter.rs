use crate::detector::Detection;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct MappingStore {
    // Map from Masked_Token -> Original_Value
    store: RwLock<HashMap<String, String>>,
}

impl MappingStore {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            store: RwLock::new(HashMap::new()),
        })
    }

    pub fn insert_mapping(&self, token: String, original: String) {
        println!("Masked: {} -> {}", original, token);
        self.store.write().insert(token, original);
    }

    pub fn get_original(&self, token: &str) -> Option<String> {
        self.store.read().get(token).cloned()
    }
}

pub struct MultiFilter {
    mapping: Arc<MappingStore>,
    mask_style: String,
}

impl MultiFilter {
    pub fn new(mapping: Arc<MappingStore>, mask_style: String) -> Self {
        Self {
            mapping,
            mask_style, // "label", "partial", "pseudonym"
        }
    }

    // Applies detections to text, generating mapping and updating the store.
    pub fn mask_text(&self, text: &str, detections: &[Detection]) -> String {
        let mut result = text.to_string();

        for det in detections {
            // Very naive replacement based on string replace instead of indexing 
            // Better to use string builder with indices for full correctness, but this works for PoC
            let token = self.generate_token(&det);
            self.mapping.insert_mapping(token.clone(), det.original_value.clone());
            result = result.replace(&det.original_value, &token);
        }

        result
    }

    // Replace the [TOKEN] back with Original in responses
    pub fn restore_text(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        let map = self.mapping.store.read().clone();
        for (token, original) in map.iter() {
            if result.contains(token) {
                result = result.replace(token, original);
                println!("Restored: {} -> {}", token, original);
            }
        }
        
        result
    }

    fn generate_token(&self, det: &Detection) -> String {
        match self.mask_style.as_str() {
            "partial" => format!("{}***", &det.original_value.chars().take(2).collect::<String>()),
            "pseudonym" => format!("ID-{}", Uuid::new_v4().to_string().chars().take(8).collect::<String>()),
            _ => {
                // Default is label
                // E.g [ISM_1a2b] to make it unique for restoration
                let short_uuid: String = Uuid::new_v4().to_string().chars().take(4).collect();
                let label = det.dtype.as_label().trim_matches(&['[', ']'][..]);
                format!("[{}_{}]", label, short_uuid)
            }
        }
    }
}
