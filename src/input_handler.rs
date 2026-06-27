use std::sync::{Arc, Mutex};
use evdev::{Device, EventType, Key};
use std::fs;

/// Handles low-level keyboard input detection via evdev
pub struct InputHandler {
    device: Arc<Mutex<Option<Device>>>,
    key_states: Arc<Mutex<[bool; 256]>>,
}

impl InputHandler {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let device_path = Self::find_keyboard_device()?;
        
        let device = Device::open(&device_path)?;
        
        println!("Opened keyboard device: {}", device_path);
        println!("Device name: {}", device.name().unwrap_or("Unknown"));
        
        let handler = InputHandler {
            device: Arc::new(Mutex::new(Some(device))),
            key_states: Arc::new(Mutex::new([false; 256])),
        };
        
        // Start a background thread to read events
        let device_clone = handler.device.clone();
        let key_states_clone = handler.key_states.clone();
        
        std::thread::spawn(move || {
            if let Ok(mut device_opt) = device_clone.lock() {
                if let Some(device) = device_opt.as_mut() {
                    // Grab exclusive access - we'll re-send all keys via xdotool
                    let _ = device.grab();
                    
                    loop {
                        match device.fetch_events() {
                            Ok(events) => {
                                for event in events {
                                    if event.event_type() == EventType::KEY {
                                        let keycode = event.code() as usize;
                                        let value = event.value();
                                        
                                        if keycode < 256 {
                                            if let Ok(mut states) = key_states_clone.lock() {
                                                states[keycode] = value != 0;
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Error reading evdev events: {}", e);
                                std::thread::sleep(std::time::Duration::from_millis(100));
                            }
                        }
                    }
                }
            }
        });
        
        Ok(handler)
    }
    
    fn find_keyboard_device() -> Result<String, Box<dyn std::error::Error>> {
        // Look for keyboard devices in /dev/input/
        let entries = fs::read_dir("/dev/input/")?;
        
        let mut candidates: Vec<(String, String, bool)> = Vec::new(); // (path, name, has_leds)
        
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("event") {
                    if let Ok(device) = Device::open(&path) {
                        let device_name = device.name().unwrap_or("Unknown").to_string();
                        
                        // Check if it's a keyboard (has KEY_A and KEY_ENTER)
                        if let Some(keys) = device.supported_keys() {
                            if keys.contains(Key::KEY_A) && keys.contains(Key::KEY_ENTER) {
                                // Check if it has LED support (main keyboard usually does)
                                let has_leds = device.supported_leds()
                                    .map(|l| l.iter().count() > 0)
                                    .unwrap_or(false);
                                
                                println!("Found keyboard: {} - {} (has_leds: {})", 
                                    path.display(), device_name, has_leds);
                                
                                candidates.push((path.to_string_lossy().to_string(), device_name, has_leds));
                            }
                        }
                    }
                }
            }
        }
        
        // Prefer keyboard with LED support (main keyboard)
        candidates.sort_by(|a, b| b.2.cmp(&a.2));
        
        if let Some((path, name, has_leds)) = candidates.first() {
            println!("✓ Selected keyboard: {} - {} (has_leds: {})", path, name, has_leds);
            return Ok(path.clone());
        }
        
        Err("No keyboard device found. Make sure you have a keyboard connected.".into())
    }
    
    pub fn poll_keys(&self) -> Result<Vec<bool>, Box<dyn std::error::Error>> {
        if let Ok(states) = self.key_states.lock() {
            Ok(states.to_vec())
        } else {
            Err("Failed to lock key states".into())
        }
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            eprintln!("Warning: Could not initialize input handler: {}", e);
            InputHandler {
                device: Arc::new(Mutex::new(None)),
                key_states: Arc::new(Mutex::new([false; 256])),
            }
        })
    }
}
