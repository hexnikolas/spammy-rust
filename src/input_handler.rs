use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::os::unix::io::AsRawFd;
use evdev::{Device, EventType, Key};
use std::fs;
extern crate libc;

/// Device info for UI selection
#[derive(Clone, Debug)]
pub struct InputDeviceInfo {
    pub path: String,
    pub name: String,
}

/// Handles low-level keyboard input detection via evdev
pub struct InputHandler {
    key_states: Arc<Mutex<[bool; 256]>>,
    device_path: String,
    stop_flag: Arc<AtomicBool>,
}

impl InputHandler {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let device_path = Self::find_keyboard_device()?;
        Self::with_device(&device_path)
    }
    
    pub fn with_device(device_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut device = Device::open(device_path)?;
        let device_name = device.name().unwrap_or("Unknown").to_string();
        
        println!("Opened keyboard device: {}", device_path);
        println!("Device name: {}", device_name);
        
        // Set non-blocking mode on the device fd
        let fd = device.as_raw_fd();
        unsafe {
            let flags = libc::fcntl(fd, libc::F_GETFL);
            libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }
        
        // Grab exclusive access
        let _ = device.grab();
        
        let stop_flag = Arc::new(AtomicBool::new(false));
        let key_states = Arc::new(Mutex::new([false; 256]));
        
        let handler = InputHandler {
            key_states: key_states.clone(),
            device_path: device_path.to_string(),
            stop_flag: stop_flag.clone(),
        };
        
        // Start a background thread to read events - device is moved into thread
        let stop_flag_clone = stop_flag.clone();
        
        std::thread::spawn(move || {
            loop {
                // Check if we should stop
                if stop_flag_clone.load(Ordering::Relaxed) {
                    println!("Input handler thread stopping...");
                    let _ = device.ungrab();
                    break;
                }
                
                // Non-blocking fetch
                match device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            if event.event_type() == EventType::KEY {
                                let keycode = event.code() as usize;
                                let value = event.value();
                                
                                if keycode < 256 {
                                    if let Ok(mut states) = key_states.lock() {
                                        states[keycode] = value != 0;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        // EAGAIN/EWOULDBLOCK means no events available (non-blocking)
                        if e.raw_os_error() != Some(libc::EAGAIN) && e.raw_os_error() != Some(libc::EWOULDBLOCK) {
                            // Real error - check stop flag and maybe log
                            if stop_flag_clone.load(Ordering::Relaxed) {
                                break;
                            }
                        }
                    }
                }
                
                // Small sleep to prevent busy-waiting
                std::thread::sleep(std::time::Duration::from_millis(2));
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
    
    pub fn get_device_path(&self) -> &str {
        &self.device_path
    }
    
    /// Stop the background thread
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }
    
    /// List all available keyboard devices
    pub fn list_available_devices() -> Vec<InputDeviceInfo> {
        let mut devices = Vec::new();
        
        if let Ok(entries) = fs::read_dir("/dev/input/") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("event") {
                        if let Ok(device) = Device::open(&path) {
                            // Check if it's a keyboard (has KEY_A)
                            if let Some(keys) = device.supported_keys() {
                                if keys.contains(Key::KEY_A) {
                                    let device_name = device.name().unwrap_or("Unknown").to_string();
                                    devices.push(InputDeviceInfo {
                                        path: path.to_string_lossy().to_string(),
                                        name: device_name,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Sort by path for consistent ordering
        devices.sort_by(|a, b| a.path.cmp(&b.path));
        devices
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            eprintln!("Warning: Could not initialize input handler: {}", e);
            InputHandler {
                key_states: Arc::new(Mutex::new([false; 256])),
                device_path: String::new(),
                stop_flag: Arc::new(AtomicBool::new(false)),
            }
        })
    }
}
