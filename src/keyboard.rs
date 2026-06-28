/// Keyboard layout and key definitions
#[derive(Clone)]
pub struct KeyboardLayout {
    pub keys: Vec<KeyInfo>,
}

#[derive(Clone, Debug)]
pub struct KeyInfo {
    pub code: u32,
    pub name: String,
    pub row: usize,
    pub width: f32,
    pub is_spacer: bool,
}

impl KeyboardLayout {
    pub fn new() -> Self {
        let keys = vec![
            // ESC + F-keys row (with gaps like a real keyboard)
            KeyInfo { code: 1, name: "ESC".to_string(), row: 0, width: 1.0, is_spacer: false },
            KeyInfo { code: 0, name: "".to_string(), row: 0, width: 0.5, is_spacer: true },
            KeyInfo { code: 59, name: "F1".to_string(), row: 0, width: 1.0, is_spacer: false },
            KeyInfo { code: 60, name: "F2".to_string(), row: 0, width: 1.0, is_spacer: false },
            KeyInfo { code: 61, name: "F3".to_string(), row: 0, width: 1.0, is_spacer: false },
            KeyInfo { code: 62, name: "F4".to_string(), row: 0, width: 1.0, is_spacer: false },
            KeyInfo { code: 0, name: "".to_string(), row: 0, width: 0.3, is_spacer: true },
            KeyInfo { code: 63, name: "F5".to_string(), row: 0, width: 1.0, is_spacer: false },
            KeyInfo { code: 64, name: "F6".to_string(), row: 0, width: 1.0, is_spacer: false },
            KeyInfo { code: 65, name: "F7".to_string(), row: 0, width: 1.0, is_spacer: false },
            KeyInfo { code: 66, name: "F8".to_string(), row: 0, width: 1.0, is_spacer: false },
            KeyInfo { code: 0, name: "".to_string(), row: 0, width: 0.3, is_spacer: true },
            KeyInfo { code: 67, name: "F9".to_string(), row: 0, width: 1.0, is_spacer: false },
            KeyInfo { code: 68, name: "F10".to_string(), row: 0, width: 1.0, is_spacer: false },
            KeyInfo { code: 87, name: "F11".to_string(), row: 0, width: 1.0, is_spacer: false },
            KeyInfo { code: 88, name: "F12".to_string(), row: 0, width: 1.0, is_spacer: false },
            
            // Number row with backtick
            KeyInfo { code: 41, name: "`".to_string(), row: 1, width: 0.9, is_spacer: false },
            KeyInfo { code: 2, name: "1".to_string(), row: 1, width: 1.0, is_spacer: false },
            KeyInfo { code: 3, name: "2".to_string(), row: 1, width: 1.0, is_spacer: false },
            KeyInfo { code: 4, name: "3".to_string(), row: 1, width: 1.0, is_spacer: false },
            KeyInfo { code: 5, name: "4".to_string(), row: 1, width: 1.0, is_spacer: false },
            KeyInfo { code: 6, name: "5".to_string(), row: 1, width: 1.0, is_spacer: false },
            KeyInfo { code: 7, name: "6".to_string(), row: 1, width: 1.0, is_spacer: false },
            KeyInfo { code: 8, name: "7".to_string(), row: 1, width: 1.0, is_spacer: false },
            KeyInfo { code: 9, name: "8".to_string(), row: 1, width: 1.0, is_spacer: false },
            KeyInfo { code: 10, name: "9".to_string(), row: 1, width: 1.0, is_spacer: false },
            KeyInfo { code: 11, name: "0".to_string(), row: 1, width: 1.0, is_spacer: false },
            KeyInfo { code: 12, name: "-".to_string(), row: 1, width: 1.0, is_spacer: false },
            KeyInfo { code: 13, name: "=".to_string(), row: 1, width: 1.0, is_spacer: false },
            KeyInfo { code: 14, name: "BKSP".to_string(), row: 1, width: 1.0, is_spacer: false },
            
            // QWERTY row
            KeyInfo { code: 15, name: "TAB".to_string(), row: 2, width: 1.2, is_spacer: false },
            KeyInfo { code: 16, name: "Q".to_string(), row: 2, width: 1.0, is_spacer: false },
            KeyInfo { code: 17, name: "W".to_string(), row: 2, width: 1.0, is_spacer: false },
            KeyInfo { code: 18, name: "E".to_string(), row: 2, width: 1.0, is_spacer: false },
            KeyInfo { code: 19, name: "R".to_string(), row: 2, width: 1.0, is_spacer: false },
            KeyInfo { code: 20, name: "T".to_string(), row: 2, width: 1.0, is_spacer: false },
            KeyInfo { code: 21, name: "Y".to_string(), row: 2, width: 1.0, is_spacer: false },
            KeyInfo { code: 22, name: "U".to_string(), row: 2, width: 1.0, is_spacer: false },
            KeyInfo { code: 23, name: "I".to_string(), row: 2, width: 1.0, is_spacer: false },
            KeyInfo { code: 24, name: "O".to_string(), row: 2, width: 1.0, is_spacer: false },
            KeyInfo { code: 25, name: "P".to_string(), row: 2, width: 1.0, is_spacer: false },
            KeyInfo { code: 26, name: "[".to_string(), row: 2, width: 1.0, is_spacer: false },
            KeyInfo { code: 27, name: "]".to_string(), row: 2, width: 0.8, is_spacer: false },
            KeyInfo { code: 43, name: "\\".to_string(), row: 2, width: 1.0, is_spacer: false },
            
            // ASDF row
            KeyInfo { code: 58, name: "CAPS".to_string(), row: 3, width: 1.5, is_spacer: false },
            KeyInfo { code: 30, name: "A".to_string(), row: 3, width: 1.0, is_spacer: false },
            KeyInfo { code: 31, name: "S".to_string(), row: 3, width: 1.0, is_spacer: false },
            KeyInfo { code: 32, name: "D".to_string(), row: 3, width: 1.0, is_spacer: false },
            KeyInfo { code: 33, name: "F".to_string(), row: 3, width: 1.0, is_spacer: false },
            KeyInfo { code: 34, name: "G".to_string(), row: 3, width: 1.0, is_spacer: false },
            KeyInfo { code: 35, name: "H".to_string(), row: 3, width: 1.0, is_spacer: false },
            KeyInfo { code: 36, name: "J".to_string(), row: 3, width: 1.0, is_spacer: false },
            KeyInfo { code: 37, name: "K".to_string(), row: 3, width: 1.0, is_spacer: false },
            KeyInfo { code: 38, name: "L".to_string(), row: 3, width: 1.0, is_spacer: false },
            KeyInfo { code: 39, name: ";".to_string(), row: 3, width: 1.0, is_spacer: false },
            KeyInfo { code: 40, name: "'".to_string(), row: 3, width: 1.0, is_spacer: false },
            KeyInfo { code: 28, name: "ENTER".to_string(), row: 3, width: 1.6, is_spacer: false },
            
            // ZXCV row
            KeyInfo { code: 42, name: "SHIFT".to_string(), row: 4, width: 2.1, is_spacer: false },
            KeyInfo { code: 44, name: "Z".to_string(), row: 4, width: 1.0, is_spacer: false },
            KeyInfo { code: 45, name: "X".to_string(), row: 4, width: 1.0, is_spacer: false },
            KeyInfo { code: 46, name: "C".to_string(), row: 4, width: 1.0, is_spacer: false },
            KeyInfo { code: 47, name: "V".to_string(), row: 4, width: 1.0, is_spacer: false },
            KeyInfo { code: 48, name: "B".to_string(), row: 4, width: 1.0, is_spacer: false },
            KeyInfo { code: 49, name: "N".to_string(), row: 4, width: 1.0, is_spacer: false },
            KeyInfo { code: 50, name: "M".to_string(), row: 4, width: 1.0, is_spacer: false },
            KeyInfo { code: 51, name: ",".to_string(), row: 4, width: 1.0, is_spacer: false },
            KeyInfo { code: 52, name: ".".to_string(), row: 4, width: 1.0, is_spacer: false },
            KeyInfo { code: 53, name: "/".to_string(), row: 4, width: 1.0, is_spacer: false },
            KeyInfo { code: 54, name: "SHIFT".to_string(), row: 4, width: 2.0, is_spacer: false },
            
            // Space bar row
            KeyInfo { code: 29, name: "CTRL".to_string(), row: 5, width: 1.2, is_spacer: false },
            KeyInfo { code: 125, name: "WIN".to_string(), row: 5, width: 1.2, is_spacer: false },
            KeyInfo { code: 56, name: "ALT".to_string(), row: 5, width: 1.2, is_spacer: false },
            KeyInfo { code: 57, name: "SPACE".to_string(), row: 5, width: 5.9, is_spacer: false },
            KeyInfo { code: 100, name: "ALT".to_string(), row: 5, width: 1.2, is_spacer: false },
            KeyInfo { code: 126, name: "WIN".to_string(), row: 5, width: 1.2, is_spacer: false },
            KeyInfo { code: 127, name: "MENU".to_string(), row: 5, width: 1.2, is_spacer: false },
            KeyInfo { code: 97, name: "CTRL".to_string(), row: 5, width: 1.2, is_spacer: false },
        ];
        
        KeyboardLayout { keys }
    }
}

impl Default for KeyboardLayout {
    fn default() -> Self {
        Self::new()
    }
}
