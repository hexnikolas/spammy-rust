use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub active_keys: HashSet<u32>,
    #[serde(default)]
    pub speedy_keys: HashSet<u32>,
    pub repeat_interval_ms: u64,
    #[serde(default)]
    pub target_window_name: Option<String>,
    #[serde(default)]
    pub input_device_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilesData {
    pub profiles: Vec<Profile>,
    pub active_profile: Option<String>,
}

impl Default for Profile {
    fn default() -> Self {
        Profile {
            name: "Default".to_string(),
            active_keys: HashSet::new(),
            speedy_keys: HashSet::new(),
            repeat_interval_ms: 100,
            target_window_name: None,
            input_device_path: None,
        }
    }
}

impl Profile {
    pub fn from_state(name: &str, active_keys: &[bool], speedy_keys: &[bool], repeat_interval_ms: u64, target_window_name: Option<String>, input_device_path: Option<String>) -> Self {
        let keys: HashSet<u32> = active_keys
            .iter()
            .enumerate()
            .filter(|(_, &active)| active)
            .map(|(idx, _)| idx as u32)
            .collect();
        
        let speedy: HashSet<u32> = speedy_keys
            .iter()
            .enumerate()
            .filter(|(_, &active)| active)
            .map(|(idx, _)| idx as u32)
            .collect();
        
        Profile {
            name: name.to_string(),
            active_keys: keys,
            speedy_keys: speedy,
            repeat_interval_ms,
            target_window_name,
            input_device_path,
        }
    }
    
    pub fn to_active_keys_vec(&self) -> Vec<bool> {
        let mut keys = vec![false; 256];
        for &code in &self.active_keys {
            if (code as usize) < keys.len() {
                keys[code as usize] = true;
            }
        }
        keys
    }
    
    pub fn to_speedy_keys_vec(&self) -> Vec<bool> {
        let mut keys = vec![false; 256];
        for &code in &self.speedy_keys {
            if (code as usize) < keys.len() {
                keys[code as usize] = true;
            }
        }
        keys
    }
}

impl Default for ProfilesData {
    fn default() -> Self {
        ProfilesData {
            profiles: vec![Profile::default()],
            active_profile: Some("Default".to_string()),
        }
    }
}

fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("spammy");
    
    fs::create_dir_all(&config_dir).ok();
    config_dir.join("profiles.json")
}

pub fn save_profiles(data: &ProfilesData) -> Result<(), String> {
    let path = get_config_path();
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize profiles: {}", e))?;
    
    fs::write(&path, json)
        .map_err(|e| format!("Failed to write profiles to {:?}: {}", path, e))?;
    
    println!("Profiles saved to {:?}", path);
    Ok(())
}

pub fn load_profiles() -> ProfilesData {
    let path = get_config_path();
    
    match fs::read_to_string(&path) {
        Ok(json) => {
            match serde_json::from_str(&json) {
                Ok(data) => {
                    println!("Profiles loaded from {:?}", path);
                    data
                }
                Err(e) => {
                    eprintln!("Failed to parse profiles: {}", e);
                    ProfilesData::default()
                }
            }
        }
        Err(_) => {
            println!("No profiles file found, using defaults");
            ProfilesData::default()
        }
    }
}
