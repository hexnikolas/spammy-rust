use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub key_bindings: HashMap<u32, KeyBinding>,
    pub repeat_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    pub keycode: u32,
    pub enabled: bool,
}

impl Default for Profile {
    fn default() -> Self {
        Profile {
            name: "Default".to_string(),
            key_bindings: HashMap::new(),
            repeat_interval_ms: 100,
        }
    }
}

impl Profile {
    pub fn new(name: &str) -> Self {
        Profile {
            name: name.to_string(),
            key_bindings: HashMap::new(),
            repeat_interval_ms: 100,
        }
    }
    
    pub fn enable_key(&mut self, keycode: u32) {
        self.key_bindings.insert(
            keycode,
            KeyBinding {
                keycode,
                enabled: true,
            },
        );
    }
    
    pub fn disable_key(&mut self, keycode: u32) {
        if let Some(binding) = self.key_bindings.get_mut(&keycode) {
            binding.enabled = false;
        }
    }
    
    pub fn is_key_enabled(&self, keycode: u32) -> bool {
        self.key_bindings
            .get(&keycode)
            .map(|b| b.enabled)
            .unwrap_or(false)
    }
}
