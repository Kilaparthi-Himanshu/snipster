use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shortcut {
    // Windows virtual-key code
    pub key: u32,
    pub win: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Default for Shortcut {
    fn default() -> Self {
        Self {
            key: 0x53, // S
            win: true,
            ctrl: true,
            alt: false,
            shift: true,
        }
    }
}

impl Shortcut {
    pub fn display(&self) -> String {
        let mut parts = Vec::new();

        // Keep modifier display order consistent.
        if self.win {
            parts.push("Win".to_string());
        }

        if self.ctrl {
            parts.push("Ctrl".to_string());
        }

        if self.alt {
            parts.push("Alt".to_string());
        }

        if self.shift {
            parts.push("Shift".to_string());
        }

        let key_name = match self.key {
            0x30..=0x39 => char::from_u32(self.key)
                .unwrap()
                .to_string(),

            0x41..=0x5A => char::from_u32(self.key)
                .unwrap()
                .to_string(),

            0x09 => "Tab".to_string(),
                0x14 => "Caps Lock".to_string(),
                0xC0 => "`".to_string(),

            0x70..=0x87 => format!("F{}", self.key - 0x6F),

            _ => format!("VK_{:02X}", self.key),
        };

        parts.push(key_name);

        parts.join(" + ")
    }
}
