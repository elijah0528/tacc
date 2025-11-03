use crate::error::TaccError;
use once_cell::sync::Lazy;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

// Entry from a mapping JSONL file
#[derive(Deserialize)]
struct MappingEntry {
    token: String,
    token_id: u32,
    mapped_id: i32,
    #[allow(dead_code)]
    count: Option<u32>,
}

// Bidirectional mapping for fast lookup
pub struct TokenMapping {
    pub token_to_mapped: FxHashMap<String, i32>,
    pub token_id_to_mapped: FxHashMap<u32, i32>,
    pub mapped_to_token_id: FxHashMap<i32, u32>,
    pub mapped_to_token: FxHashMap<i32, String>,
}

// Caches mapping per tokenizer (thread-safe, lazy)
static MAPPING_CACHE: Lazy<Mutex<FxHashMap<String, Arc<TokenMapping>>>> =
    Lazy::new(|| Mutex::new(FxHashMap::default()));

impl TokenMapping {
    /// Load mapping for a tokenizer (from cache or embedded file)
    pub fn load(tokenizer: &str) -> Result<Arc<Self>, TaccError> {
        {
            let cache = MAPPING_CACHE.lock().unwrap();
            if let Some(mapping) = cache.get(tokenizer) {
                return Ok(Arc::clone(mapping));
            }
        }
        // Not cached: load from embedded file
        let data = match tokenizer {
            "cl100k_base" => include_str!("data/cl100k_base.jsonl"),
            "gpt2" => include_str!("data/gpt2.jsonl"),
            "o200k_base" => include_str!("data/o200k_base.jsonl"),
            "o200k_harmony" => include_str!("data/o200k_harmony.jsonl"),
            "p50k_base" => include_str!("data/p50k_base.jsonl"),
            "p50k_edit" => include_str!("data/p50k_edit.jsonl"),
            "r50k_base" => include_str!("data/r50k_base.jsonl"),
            _ => {
                return Err(TaccError::MappingNotFound {
                    tokenizer: tokenizer.to_string(),
                    available: list_available_tokenizers(),
                })
            }
        };

        let mapping = Self::parse_jsonl(data)?;
        let arc_mapping = Arc::new(mapping);

        let mut cache = MAPPING_CACHE.lock().unwrap();
        cache.insert(tokenizer.to_string(), Arc::clone(&arc_mapping));
        Ok(arc_mapping)
    }

    // Parse JSONL to build lookup maps
    fn parse_jsonl(data: &str) -> Result<Self, TaccError> {
        let mut token_to_mapped = FxHashMap::default();
        let mut token_id_to_mapped = FxHashMap::default();
        let mut mapped_to_token_id = FxHashMap::default();
        let mut mapped_to_token = FxHashMap::default();

        for line in data.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let entry: MappingEntry = serde_json::from_str(trimmed)?;
            token_to_mapped.insert(entry.token.clone(), entry.mapped_id);
            token_id_to_mapped.insert(entry.token_id, entry.mapped_id);
            mapped_to_token_id.insert(entry.mapped_id, entry.token_id);
            mapped_to_token.insert(entry.mapped_id, entry.token);
        }

        Ok(Self {
            token_to_mapped,
            token_id_to_mapped,
            mapped_to_token_id,
            mapped_to_token,
        })
    }

    pub fn token_id_to_mapped_id(&self, token_id: u32) -> Result<i32, TaccError> {
        self.token_id_to_mapped
            .get(&token_id)
            .copied()
            .ok_or(TaccError::TokenIdNotFound(token_id))
    }

    pub fn mapped_id_to_token_id(&self, mapped_id: i32) -> Result<u32, TaccError> {
        self.mapped_to_token_id
            .get(&mapped_id)
            .copied()
            .ok_or(TaccError::MappedIdNotFound(mapped_id))
    }

    pub fn token_to_mapped_id(&self, token: &str) -> Result<i32, TaccError> {
        self.token_to_mapped
            .get(token)
            .copied()
            .ok_or(TaccError::TokenNotFound(token.to_string()))
    }

    pub fn mapped_id_to_token(&self, mapped_id: i32) -> Result<&str, TaccError> {
        self.mapped_to_token
            .get(&mapped_id)
            .map(|s| s.as_str())
            .ok_or(TaccError::MappedIdNotFound(mapped_id))
    }
}

// List of supported tokenizers
pub fn list_available_tokenizers() -> Vec<String> {
    vec![
        "cl100k_base".to_string(),
        "gpt2".to_string(),
        "o200k_base".to_string(),
        "o200k_harmony".to_string(),
        "p50k_base".to_string(),
        "p50k_edit".to_string(),
        "r50k_base".to_string(),
    ]
}
