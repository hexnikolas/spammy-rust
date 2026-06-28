use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::keyboard::KeyboardLayout;
use crate::profile::{Profile, ProfilesData, save_profiles, load_profiles};
use crate::input_handler::{InputHandler, InputDeviceInfo};
use crate::key_sender::KeySender;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref GLOBAL_KEY_SENDER: Mutex<Option<Arc<KeySender>>> = Mutex::new(None);
}

pub struct SpammyApp {
    enabled: bool,
    keyboard_layout: KeyboardLayout,
    profiles_data: ProfilesData,
    active_keys: Vec<bool>,
    speedy_keys: Vec<bool>,
    pressed_keys: Vec<bool>,
    prev_pressed_keys: Vec<bool>,
    prev_speedy_pressed: Vec<bool>,
    last_key_send: Instant,
    key_repeat_interval: Duration,
    input_handler: Option<Arc<Mutex<InputHandler>>>,
    key_sender: Option<Arc<KeySender>>,
    target_window_id: Option<u32>,
    target_window_name: Option<String>,
    available_windows: Vec<(u32, String)>,
    show_window_picker: bool,
    new_profile_name: String,
    available_input_devices: Vec<InputDeviceInfo>,
    current_input_device: Option<String>,
}

impl SpammyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Load profiles first
        let profiles_data = load_profiles();
        
        // Get initial state from active profile
        let (active_keys, speedy_keys, interval, target_window, saved_input_device) = if let Some(ref name) = profiles_data.active_profile {
            if let Some(profile) = profiles_data.profiles.iter().find(|p| &p.name == name) {
                (profile.to_active_keys_vec(), profile.to_speedy_keys_vec(), profile.repeat_interval_ms, profile.target_window_name.clone(), profile.input_device_path.clone())
            } else {
                (vec![false; 256], vec![false; 256], 100, None, None)
            }
        } else {
            (vec![false; 256], vec![false; 256], 100, None, None)
        };
        
        let mut app = SpammyApp {
            enabled: true,
            keyboard_layout: KeyboardLayout::new(),
            profiles_data,
            active_keys,
            speedy_keys,
            pressed_keys: vec![false; 256],
            prev_pressed_keys: vec![false; 256],
            prev_speedy_pressed: vec![false; 256],
            last_key_send: Instant::now(),
            key_repeat_interval: Duration::from_millis(interval),
            input_handler: None,
            key_sender: None,
            target_window_id: None,
            target_window_name: None,
            available_windows: Vec::new(),
            show_window_picker: false,
            new_profile_name: String::new(),
            available_input_devices: Vec::new(),
            current_input_device: saved_input_device,
        };
        
        // Restore target window after app is created
        if let Some(ref window_name) = target_window {
            app.set_target_window_by_name(window_name);
        }
        
        app.initialize();
        app
    }
    
    fn initialize(&mut self) {
        // List available input devices
        self.available_input_devices = InputHandler::list_available_devices();
        
        // Initialize input handler (requires sudo/proper permissions)
        // Use saved device path if available, otherwise auto-detect
        let init_result = if let Some(ref saved_path) = self.current_input_device {
            println!("Restoring saved input device: {}", saved_path);
            InputHandler::with_device(saved_path)
        } else {
            InputHandler::new()
        };
        
        match init_result {
            Ok(handler) => {
                self.current_input_device = Some(handler.get_device_path().to_string());
                self.input_handler = Some(Arc::new(Mutex::new(handler)));
                println!("Input handler initialized");
            }
            Err(e) => {
                eprintln!("Failed to initialize input handler: {}", e);
                eprintln!("Keyboard strokes won't be detected");
                // If saved device failed, try auto-detect as fallback
                if self.current_input_device.is_some() {
                    println!("Trying auto-detect as fallback...");
                    self.current_input_device = None;
                    if let Ok(handler) = InputHandler::new() {
                        self.current_input_device = Some(handler.get_device_path().to_string());
                        self.input_handler = Some(Arc::new(Mutex::new(handler)));
                        println!("Input handler initialized with fallback device");
                    }
                }
            }
        }
        
        // Initialize key sender
        match KeySender::new() {
            Ok(sender) => {
                let sender_arc = Arc::new(sender);
                self.key_sender = Some(sender_arc.clone());
                
                // Store in global for signal handler
                if let Ok(mut global) = GLOBAL_KEY_SENDER.lock() {
                    *global = Some(sender_arc);
                }
                
                println!("Key sender initialized");
            }
            Err(e) => {
                eprintln!("Failed to initialize key sender: {}", e);
                eprintln!("Key sending won't work");
            }
        }
    }
    
    // Profile management
    pub fn get_profile_names(&self) -> Vec<String> {
        self.profiles_data.profiles.iter().map(|p| p.name.clone()).collect()
    }
    
    pub fn get_active_profile_name(&self) -> Option<&str> {
        self.profiles_data.active_profile.as_deref()
    }
    
    pub fn select_profile(&mut self, name: &str) {
        // Clone data first to avoid borrow issues
        let profile_data = self.profiles_data.profiles.iter()
            .find(|p| p.name == name)
            .map(|p| (p.to_active_keys_vec(), p.to_speedy_keys_vec(), p.repeat_interval_ms, p.target_window_name.clone(), p.input_device_path.clone()));
        
        if let Some((keys, speedy, interval, target_window, input_device)) = profile_data {
            self.active_keys = keys;
            self.speedy_keys = speedy;
            self.prev_speedy_pressed = vec![false; 256];
            self.key_repeat_interval = Duration::from_millis(interval);
            self.profiles_data.active_profile = Some(name.to_string());
            
            // Restore target window by name
            if let Some(window_name) = target_window {
                self.set_target_window_by_name(&window_name);
            } else {
                self.clear_target_window();
            }
            
            // Restore input device
            if let Some(device_path) = input_device {
                if self.current_input_device.as_deref() != Some(&device_path) {
                    self.switch_input_device(&device_path);
                }
            }
        }
    }
    
    pub fn save_current_as_profile(&mut self, name: &str) {
        println!("Saving profile: '{}'", name);
        let profile = Profile::from_state(
            name, 
            &self.active_keys,
            &self.speedy_keys,
            self.key_repeat_interval.as_millis() as u64,
            self.target_window_name.clone(),
            self.current_input_device.clone(),
        );
        
        // Update or add profile
        if let Some(existing) = self.profiles_data.profiles.iter_mut().find(|p| p.name == name) {
            println!("Updating existing profile");
            *existing = profile;
        } else {
            println!("Adding new profile");
            self.profiles_data.profiles.push(profile);
        }
        
        self.profiles_data.active_profile = Some(name.to_string());
        println!("Total profiles: {}", self.profiles_data.profiles.len());
        
        if let Err(e) = save_profiles(&self.profiles_data) {
            eprintln!("Failed to save profiles: {}", e);
        }
    }
    
    pub fn delete_profile(&mut self, name: &str) {
        // Don't delete the last profile
        if self.profiles_data.profiles.len() <= 1 {
            return;
        }
        
        self.profiles_data.profiles.retain(|p| p.name != name);
        
        // If we deleted the active profile, switch to the first one
        if self.profiles_data.active_profile.as_deref() == Some(name) {
            if let Some(first) = self.profiles_data.profiles.first() {
                self.select_profile(&first.name.clone());
            }
        }
        
        if let Err(e) = save_profiles(&self.profiles_data) {
            eprintln!("Failed to save profiles: {}", e);
        }
    }
    
    pub fn get_new_profile_name(&self) -> &str {
        &self.new_profile_name
    }
    
    pub fn set_new_profile_name(&mut self, name: String) {
        self.new_profile_name = name;
    }
    
    pub fn toggle_key(&mut self, key_index: usize) {
        if key_index < self.active_keys.len() {
            // If enabling spammy mode, disable speedy mode for this key
            if !self.active_keys[key_index] && key_index < self.speedy_keys.len() {
                self.speedy_keys[key_index] = false;
            }
            self.active_keys[key_index] = !self.active_keys[key_index];
        }
    }
    
    pub fn toggle_speedy_key(&mut self, key_index: usize) {
        if key_index < self.speedy_keys.len() {
            // If enabling speedy mode, disable spammy mode for this key
            if !self.speedy_keys[key_index] && key_index < self.active_keys.len() {
                self.active_keys[key_index] = false;
            }
            self.speedy_keys[key_index] = !self.speedy_keys[key_index];
        }
    }
    
    pub fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    pub fn update(&mut self) {
        // Update pressed keys from input handler
        if let Some(handler) = &self.input_handler {
            if let Ok(h) = handler.lock() {
                if let Ok(pressed) = h.poll_keys() {
                    self.pressed_keys = pressed;
                }
            }
        }
        
        // Send ALL pressed keys every frame (normal keyboard behavior)
        self.send_all_pressed_keys();
        
        // Only spam active keys when enabled
        if !self.enabled {
            return;
        }
        
        // On timer, spam only the active keys
        let now = Instant::now();
        if now.duration_since(self.last_key_send) >= self.key_repeat_interval {
            self.spam_active_keys();
            self.last_key_send = now;
        }
        
        // Handle speedy keys (single-tap on press)
        self.handle_speedy_keys();
    }
    
    fn send_all_pressed_keys(&mut self) {
        if let Some(sender) = &self.key_sender {
            // Send keys on TRANSITION (when they go from not-pressed to pressed)
            for code in 0..256u32 {
                let code_idx = code as usize;
                let is_pressed = code_idx < self.pressed_keys.len() && self.pressed_keys[code_idx];
                let was_pressed = code_idx < self.prev_pressed_keys.len() && self.prev_pressed_keys[code_idx];
                
                // Key pressed (transition from not-pressed to pressed)
                if is_pressed && !was_pressed {
                    if let Err(_e) = sender.key_down(code) {
                        // Silently ignore errors for unmapped keys
                    }
                }
                // Key released (transition from pressed to not-pressed)
                else if !is_pressed && was_pressed {
                    if let Err(_e) = sender.key_up(code) {
                        // Silently ignore errors for unmapped keys
                    }
                }
            }
        }
        
        // Update previous state for next frame
        self.prev_pressed_keys = self.pressed_keys.clone();
    }
    
    fn spam_active_keys(&mut self) {
        // Only spam if target window is focused (or if no target window is set)
        if let Some(_) = self.target_window_id {
            if !self.is_target_window_focused() {
                return;
            }
        }
        
        if let Some(sender) = &self.key_sender {
            // Collect all keys that should be spammed (active AND currently pressed)
            let keys_to_spam: Vec<u32> = (0..256u32)
                .filter(|&code| {
                    let idx = code as usize;
                    idx < self.active_keys.len() && self.active_keys[idx]
                        && idx < self.pressed_keys.len() && self.pressed_keys[idx]
                })
                .collect();
            
            // Send all keys in a single xdotool call
            if !keys_to_spam.is_empty() {
                if let Err(_e) = sender.send_keys_batch(&keys_to_spam) {
                    // Silently ignore errors
                }
            }
        }
    }
    
    fn handle_speedy_keys(&mut self) {
        if let Some(sender) = &self.key_sender {
            for code in 0..256u32 {
                let code_idx = code as usize;
                if code_idx >= self.speedy_keys.len() || !self.speedy_keys[code_idx] {
                    continue;
                }
                let is_pressed = code_idx < self.pressed_keys.len() && self.pressed_keys[code_idx];
                let was_pressed = code_idx < self.prev_speedy_pressed.len() && self.prev_speedy_pressed[code_idx];
                if is_pressed && !was_pressed {
                    if let Err(_e) = sender.send_key(code) {
                        // Silently ignore errors
                    }
                }
            }
        }
        self.prev_speedy_pressed = self.pressed_keys.clone();
    }
    

    pub fn get_active_keys(&self) -> &[bool] {
        &self.active_keys
    }
    
    pub fn get_pressed_keys(&self) -> &[bool] {
        &self.pressed_keys
    }
    
    pub fn get_speedy_keys(&self) -> &[bool] {
        &self.speedy_keys
    }
    
    pub fn get_keyboard_layout(&self) -> &KeyboardLayout {
        &self.keyboard_layout
    }
    
    pub fn get_repeat_interval_ms(&self) -> u64 {
        self.key_repeat_interval.as_millis() as u64
    }
    
    pub fn set_repeat_interval_ms(&mut self, ms: u64) {
        if ms > 0 {
            self.key_repeat_interval = Duration::from_millis(ms);
        }
    }
    
    pub fn refresh_window_list(&mut self) {
        self.available_windows.clear();
        
        // Get all window IDs
        if let Ok(output) = std::process::Command::new("xdotool")
            .arg("search")
            .arg("--onlyvisible")
            .arg("--class")
            .arg(".")
            .output() {
            if let Ok(window_ids_str) = String::from_utf8(output.stdout) {
                for line in window_ids_str.lines() {
                    if let Ok(window_id) = line.trim().parse::<u32>() {
                        // Get window name
                        if let Ok(name_output) = std::process::Command::new("xdotool")
                            .arg("getwindowname")
                            .arg(window_id.to_string())
                            .output() {
                            if let Ok(name) = String::from_utf8(name_output.stdout) {
                                let name = name.trim().to_string();
                                if !name.is_empty() {
                                    self.available_windows.push((window_id, name));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    pub fn set_target_window_by_id(&mut self, window_id: u32) {
        self.target_window_id = Some(window_id);
        
        // Get the window name
        if let Ok(name_output) = std::process::Command::new("xdotool")
            .arg("getwindowname")
            .arg(window_id.to_string())
            .output() {
            if let Ok(name) = String::from_utf8(name_output.stdout) {
                self.target_window_name = Some(name.trim().to_string());
                println!("✓ Target window set: {} (ID: {})", self.target_window_name.as_ref().unwrap(), window_id);
            }
        }
        
        self.show_window_picker = false;
    }
    
    pub fn set_target_window_by_name(&mut self, name: &str) {
        // Search for window by name using xdotool
        if let Ok(output) = std::process::Command::new("xdotool")
            .arg("search")
            .arg("--name")
            .arg(name)
            .output() {
            if let Ok(ids_str) = String::from_utf8(output.stdout) {
                // Get the first matching window ID
                if let Some(id_str) = ids_str.lines().next() {
                    if let Ok(window_id) = id_str.trim().parse::<u32>() {
                        self.target_window_id = Some(window_id);
                        self.target_window_name = Some(name.to_string());
                        println!("✓ Target window restored: {} (ID: {})", name, window_id);
                        return;
                    }
                }
            }
        }
        // Window not found - keep the name but clear the ID
        self.target_window_name = Some(name.to_string());
        self.target_window_id = None;
        println!("⚠ Target window '{}' not found (may need to open it)", name);
    }
    
    pub fn toggle_window_picker(&mut self) {
        self.show_window_picker = !self.show_window_picker;
        if self.show_window_picker {
            self.refresh_window_list();
        }
    }
    
    pub fn get_available_windows(&self) -> &[(u32, String)] {
        &self.available_windows
    }
    
    pub fn is_window_picker_open(&self) -> bool {
        self.show_window_picker
    }
    
    pub fn close_window_picker(&mut self) {
        self.show_window_picker = false;
    }
    
    pub fn clear_target_window(&mut self) {
        self.target_window_id = None;
        self.target_window_name = None;
        println!("✓ Target window cleared");
    }
    
    pub fn get_target_window_name(&self) -> Option<&str> {
        self.target_window_name.as_deref()
    }
    
    fn is_target_window_focused(&self) -> bool {
        if let Some(target_id) = self.target_window_id {
            if let Ok(output) = std::process::Command::new("xdotool")
                .arg("getactivewindow")
                .output() {
                if let Ok(active_str) = String::from_utf8(output.stdout) {
                    if let Ok(active_id) = active_str.trim().parse::<u32>() {
                        return active_id == target_id;
                    }
                }
            }
            false
        } else {
            true // If no target window is set, always allow spamming
        }
    }
    
    pub fn get_available_input_devices(&self) -> &[InputDeviceInfo] {
        &self.available_input_devices
    }
    
    pub fn get_current_input_device(&self) -> Option<&str> {
        self.current_input_device.as_deref()
    }
    
    pub fn switch_input_device(&mut self, device_path: &str) {
        println!("Switching to input device: {}", device_path);
        
        // Stop the old handler
        if let Some(ref old_handler) = self.input_handler {
            if let Ok(handler) = old_handler.lock() {
                handler.stop();
            }
        }
        // Give the thread a moment to stop and ungrab
        std::thread::sleep(std::time::Duration::from_millis(50));
        self.input_handler = None;
        
        // Create new handler for the selected device
        match InputHandler::with_device(device_path) {
            Ok(handler) => {
                self.current_input_device = Some(device_path.to_string());
                self.input_handler = Some(Arc::new(Mutex::new(handler)));
                println!("✓ Switched to device: {}", device_path);
            }
            Err(e) => {
                eprintln!("Failed to switch to device {}: {}", device_path, e);
            }
        }
    }
}

impl Drop for SpammyApp {
    fn drop(&mut self) {
        // Release all held keys when the app is shutting down
        if let Some(sender) = &self.key_sender {
            let _ = sender.release_all_keys();
        }
    }
}

impl eframe::App for SpammyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        SpammyApp::update(self);
        
        crate::ui::draw_ui(ctx, self);
    }
}
