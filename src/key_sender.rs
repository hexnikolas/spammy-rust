use std::process::Command;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use lazy_static::lazy_static;

/// Handles sending synthetic keyboard events
pub struct KeySender {
    use_xdotool: bool,
    held_keys: Arc<Mutex<HashSet<u32>>>,
}

lazy_static! {
    static ref SEND_LOCK: Mutex<()> = Mutex::new(());
}

impl KeySender {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Check if xdotool is available
        let xdotool_available = Command::new("which")
            .arg("xdotool")
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
        
        if xdotool_available {
            println!("✓ Using xdotool for key sending");
            Ok(KeySender { 
                use_xdotool: true,
                held_keys: Arc::new(Mutex::new(HashSet::new())),
            })
        } else {
            eprintln!("✗ xdotool not found. Install with: sudo apt install xdotool");
            Err("xdotool not available".into())
        }
    }
    
    /// Send a key press (without release) - for holding modifier keys
    pub fn key_down(&self, keycode: u32) -> Result<(), Box<dyn std::error::Error>> {
        if keycode == 0 || keycode > 255 {
            return Err(format!("Invalid keycode: {}", keycode).into());
        }
        
        let _lock = SEND_LOCK.lock();
        
        if let Ok(mut held) = self.held_keys.lock() {
            if !held.contains(&keycode) {
                if self.use_xdotool {
                    // Always track the key before trying to send, in case xdotool fails
                    // We still want to know about it so we can release it
                    held.insert(keycode);
                    
                    // Now try to actually send it
                    if let Err(e) = self.key_down_xdotool(keycode) {
                        eprintln!("Warning: key_down failed for {}: {}, but still tracking for release", keycode, e);
                        // Don't return error - we're still tracking this key for release_all_keys()
                        // return Err(e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Send a key release - for releasing modifier keys
    pub fn key_up(&self, keycode: u32) -> Result<(), Box<dyn std::error::Error>> {
        if keycode == 0 || keycode > 255 {
            return Err(format!("Invalid keycode: {}", keycode).into());
        }
        
        let _lock = SEND_LOCK.lock();
        
        if let Ok(mut held) = self.held_keys.lock() {
            if held.contains(&keycode) {
                if self.use_xdotool {
                    // Try to release it
                    if let Err(e) = self.key_up_xdotool(keycode) {
                        eprintln!("Warning: key_up failed for {}: {}", keycode, e);
                    }
                    // Always remove from tracking, even if xdotool failed
                    held.remove(&keycode);
                }
            }
        }
        
        Ok(())
    }
    
    /// Send a key press and release
    pub fn send_key(&self, keycode: u32) -> Result<(), Box<dyn std::error::Error>> {
        // Validate keycode
        if keycode == 0 || keycode > 255 {
            return Err(format!("Invalid keycode: {}", keycode).into());
        }
        
        if self.use_xdotool {
            self.send_key_xdotool(keycode)?;
        }
        
        Ok(())
    }
    
    fn key_down_xdotool(&self, keycode: u32) -> Result<(), Box<dyn std::error::Error>> {
        let key_name = Self::keycode_to_xdotool_name(keycode);
        
        if let Some(name) = key_name {
            let output = Command::new("xdotool")
                .arg("keydown")
                .arg(name)
                .output()?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("xdotool keydown error: {}", stderr).into());
            }
        } else {
            return Err(format!("Unknown keycode: {} (not in mapping)", keycode).into());
        }
        
        Ok(())
    }
    
    fn key_up_xdotool(&self, keycode: u32) -> Result<(), Box<dyn std::error::Error>> {
        let key_name = Self::keycode_to_xdotool_name(keycode);
        
        if let Some(name) = key_name {
            let output = Command::new("xdotool")
                .arg("keyup")
                .arg(name)
                .output()?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("xdotool keyup error: {}", stderr).into());
            }
        } else {
            return Err(format!("Unknown keycode: {} (not in mapping)", keycode).into());
        }
        
        Ok(())
    }
    
    /// Send multiple keys in a single xdotool call (much more efficient)
    pub fn send_keys_batch(&self, keycodes: &[u32]) -> Result<(), Box<dyn std::error::Error>> {
        if keycodes.is_empty() {
            return Ok(());
        }
        
        let _lock = SEND_LOCK.lock();
        
        // Collect all valid key names
        let key_names: Vec<&str> = keycodes
            .iter()
            .filter_map(|&code| Self::keycode_to_xdotool_name(code))
            .collect();
        
        if key_names.is_empty() {
            return Ok(());
        }
        
        // Send all keys in one xdotool call
        let output = Command::new("xdotool")
            .arg("key")
            .args(&key_names)
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("xdotool error: {}", stderr).into());
        }
        
        Ok(())
    }
    
    fn send_key_xdotool(&self, keycode: u32) -> Result<(), Box<dyn std::error::Error>> {
        // Lock to prevent concurrent sends
        let _lock = SEND_LOCK.lock();
        
        // Map evdev keycode to xdotool key name
        let key_name = Self::keycode_to_xdotool_name(keycode);
        
        if let Some(name) = key_name {
            let output = Command::new("xdotool")
                .arg("key")
                .arg(name)
                .output()?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("xdotool error: {}", stderr).into());
            }
        } else {
            return Err(format!("Unknown keycode: {} (not in mapping)", keycode).into());
        }
        
        Ok(())
    }
    
    fn keycode_to_xdotool_name(keycode: u32) -> Option<&'static str> {
        // Map evdev keycodes to xdotool key names
        match keycode {
            1 => Some("Escape"),
            2 => Some("1"), 3 => Some("2"), 4 => Some("3"), 5 => Some("4"),
            6 => Some("5"), 7 => Some("6"), 8 => Some("7"), 9 => Some("8"),
            10 => Some("9"), 11 => Some("0"),
            12 => Some("minus"), 13 => Some("equal"),
            14 => Some("BackSpace"),
            15 => Some("Tab"),
            16 => Some("q"), 17 => Some("w"), 18 => Some("e"), 19 => Some("r"),
            20 => Some("t"), 21 => Some("y"), 22 => Some("u"), 23 => Some("i"),
            24 => Some("o"), 25 => Some("p"),
            26 => Some("bracketleft"), 27 => Some("bracketright"),
            28 => Some("Return"),
            29 => Some("Control_L"),
            30 => Some("a"), 31 => Some("s"), 32 => Some("d"), 33 => Some("f"),
            34 => Some("g"), 35 => Some("h"), 36 => Some("j"), 37 => Some("k"),
            38 => Some("l"), 39 => Some("semicolon"), 40 => Some("apostrophe"),
            41 => Some("grave"),
            42 => Some("Shift_L"),
            43 => Some("backslash"),
            44 => Some("z"), 45 => Some("x"), 46 => Some("c"), 47 => Some("v"),
            48 => Some("b"), 49 => Some("n"), 50 => Some("m"),
            51 => Some("comma"), 52 => Some("period"), 53 => Some("slash"),
            54 => Some("Shift_R"),
            55 => Some("KP_Multiply"),
            56 => Some("Alt_L"),
            57 => Some("space"),
            58 => Some("Caps_Lock"),
            59 => Some("F1"), 60 => Some("F2"), 61 => Some("F3"),
            62 => Some("F4"), 63 => Some("F5"), 64 => Some("F6"),
            65 => Some("F7"), 66 => Some("F8"), 67 => Some("F9"),
            68 => Some("F10"),
            69 => Some("Num_Lock"),
            70 => Some("Scroll_Lock"),
            71 => Some("KP_7"),
            72 => Some("KP_8"),
            73 => Some("KP_9"),
            74 => Some("KP_Subtract"),
            75 => Some("KP_4"),
            76 => Some("KP_5"),
            77 => Some("KP_6"),
            78 => Some("KP_Add"),
            79 => Some("KP_1"),
            80 => Some("KP_2"),
            81 => Some("KP_3"),
            82 => Some("KP_0"),
            83 => Some("KP_Decimal"),
            85 => Some("Begin"),
            86 => Some("less"),
            87 => Some("F11"), 88 => Some("F12"),
            91 => Some("Super_L"),
            92 => Some("Super_R"),
            93 => Some("Menu"),
            97 => Some("Control_R"),
            99 => Some("Print"),
            100 => Some("Alt_R"),
            103 => Some("Up"),
            105 => Some("Left"),
            106 => Some("Right"),
            107 => Some("End"),
            108 => Some("Down"),
            110 => Some("Home"),
            111 => Some("Delete"),
            112 => Some("Prior"),
            113 => Some("Left"),
            114 => Some("Right"),
            115 => Some("End"),
            116 => Some("Down"),
            117 => Some("Next"),
            118 => Some("Insert"),
            96 => Some("KP_Enter"),
            119 => Some("Pause"),
            125 => Some("Super_L"),
            126 => Some("Hyper_R"),
            _ => None,
        }
    }
    
    pub fn release_all_keys(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut held) = self.held_keys.lock() {
            let keys_to_release: Vec<u32> = held.iter().copied().collect();
            eprintln!("Emergency releasing {} held keys", keys_to_release.len());
            for keycode in keys_to_release {
                let _ = self.key_up(keycode);
            }
            held.clear();
        }
        Ok(())
    }
    
    /// Force release ALL keys immediately using xdotool keyup for common keys
    pub fn force_release_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        let common_modifiers = vec![29, 42, 54, 56, 97, 100]; // ctrl, shift(L/R), alt(L/R)
        
        eprintln!("FORCE RELEASING ALL KEYS");
        
        // Release all tracked keys
        if let Ok(mut held) = self.held_keys.lock() {
            for &keycode in held.iter() {
                if let Some(name) = Self::keycode_to_xdotool_name(keycode) {
                    let _ = Command::new("xdotool")
                        .arg("keyup")
                        .arg(name)
                        .output();
                }
            }
            held.clear();
        }
        
        // Also release common modifier keys that might be stuck
        for code in common_modifiers {
            if let Some(name) = Self::keycode_to_xdotool_name(code) {
                let _ = Command::new("xdotool")
                    .arg("keyup")
                    .arg(name)
                    .output();
            }
        }
        
        Ok(())
    }
}
