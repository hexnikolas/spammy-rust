use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::keyboard::KeyboardLayout;
use crate::profile::Profile;
use crate::input_handler::InputHandler;
use crate::key_sender::KeySender;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref GLOBAL_KEY_SENDER: Mutex<Option<Arc<KeySender>>> = Mutex::new(None);
}

pub struct SpammyApp {
    enabled: bool,
    keyboard_layout: KeyboardLayout,
    profiles: HashMap<String, Profile>,
    active_profile: Option<String>,
    active_keys: Vec<bool>,
    pressed_keys: Vec<bool>,
    prev_pressed_keys: Vec<bool>,
    last_key_send: Instant,
    key_repeat_interval: Duration,
    input_handler: Option<Arc<Mutex<InputHandler>>>,
    key_sender: Option<Arc<KeySender>>,
    show_debug: bool,
    target_window_id: Option<u32>,
    target_window_name: Option<String>,
    available_windows: Vec<(u32, String)>,
    show_window_picker: bool,
}

impl SpammyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = SpammyApp {
            enabled: true,
            keyboard_layout: KeyboardLayout::new(),
            profiles: HashMap::new(),
            active_profile: None,
            active_keys: vec![false; 256],
            pressed_keys: vec![false; 256],
            prev_pressed_keys: vec![false; 256],
            last_key_send: Instant::now(),
            key_repeat_interval: Duration::from_millis(100),
            input_handler: None,
            key_sender: None,
            show_debug: false,
            target_window_id: None,
            target_window_name: None,
            available_windows: Vec::new(),
            show_window_picker: false,
        };
        
        app.initialize();
        app
    }
    
    fn initialize(&mut self) {
        self.load_profiles();
        
        // Initialize input handler (requires sudo/proper permissions)
        match InputHandler::new() {
            Ok(handler) => {
                self.input_handler = Some(Arc::new(Mutex::new(handler)));
                println!("Input handler initialized");
            }
            Err(e) => {
                eprintln!("Failed to initialize input handler: {}", e);
                eprintln!("Keyboard strokes won't be detected");
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
    
    fn load_profiles(&mut self) {
        // Load profiles from JSON config file
        let default_profile = Profile::default();
        self.profiles.insert("Default".to_string(), default_profile);
        self.active_profile = Some("Default".to_string());
    }
    
    pub fn toggle_key(&mut self, key_index: usize) {
        if key_index < self.active_keys.len() {
            self.active_keys[key_index] = !self.active_keys[key_index];
        }
    }
    
    pub fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    pub fn update(&mut self) {
        if !self.enabled {
            return;
        }
        
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
        
        // On timer, spam only the active keys
        let now = Instant::now();
        if now.duration_since(self.last_key_send) >= self.key_repeat_interval {
            self.spam_active_keys();
            self.last_key_send = now;
        }
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
                    if let Err(e) = sender.key_down(code) {
                        // Silently ignore errors for unmapped keys
                        eprintln!("Failed to press key {}: {}", code, e);
                    }
                }
                // Key released (transition from pressed to not-pressed)
                else if !is_pressed && was_pressed {
                    if let Err(e) = sender.key_up(code) {
                        // Silently ignore errors for unmapped keys
                        eprintln!("Failed to release key {}: {}", code, e);
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
            // Spam all active keys that are currently pressed
            for code in 0..256u32 {
                let code_idx = code as usize;
                if code_idx < self.active_keys.len() && self.active_keys[code_idx] 
                    && code_idx < self.pressed_keys.len() && self.pressed_keys[code_idx] {
                    if let Err(_e) = sender.send_key(code) {
                        // Silently ignore errors for unmapped keys
                    }
                }
            }
        }
    }
    

    pub fn get_active_keys(&self) -> &[bool] {
        &self.active_keys
    }
    
    pub fn get_pressed_keys(&self) -> &[bool] {
        &self.pressed_keys
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
    
    pub fn set_target_window(&mut self) {
        // Get the currently focused window using xdotool
        if let Ok(output) = std::process::Command::new("xdotool")
            .arg("getactivewindow")
            .output() {
            if let Ok(window_id_str) = String::from_utf8(output.stdout) {
                if let Ok(window_id) = window_id_str.trim().parse::<u32>() {
                    self.target_window_id = Some(window_id);
                    
                    // Try to get the window name
                    if let Ok(name_output) = std::process::Command::new("xdotool")
                        .arg("getwindowname")
                        .arg(window_id.to_string())
                        .output() {
                        if let Ok(name) = String::from_utf8(name_output.stdout) {
                            self.target_window_name = Some(name.trim().to_string());
                            println!("✓ Target window set: {} (ID: {})", self.target_window_name.as_ref().unwrap(), window_id);
                        }
                    }
                }
            }
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
