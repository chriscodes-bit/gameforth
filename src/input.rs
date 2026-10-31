use std::collections::HashSet;

/// A snapshot of the keyboard and mouse for one frame.
#[derive(Debug, Clone)]
pub struct InputState {
    pub keys_down: HashSet<i32>,
    pub keys_pressed: HashSet<i32>,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub mouse_wheel: i32,
    pub mouse_buttons_down: [bool; 8],
    pub mouse_buttons_pressed: [bool; 8],
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            mouse_x: 0,
            mouse_y: 0,
            mouse_wheel: 0,
            mouse_buttons_down: [false; 8],
            mouse_buttons_pressed: [false; 8],
        }
    }
}

impl InputState {
    pub fn key_down(&self, key: i32) -> bool {
        self.keys_down.contains(&key)
    }

    pub fn key_pressed(&self, key: i32) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn mouse_down(&self, button: i32) -> bool {
        self.mouse_buttons_down
            .get(button as usize)
            .copied()
            .unwrap_or(false)
    }

    pub fn mouse_pressed(&self, button: i32) -> bool {
        self.mouse_buttons_pressed
            .get(button as usize)
            .copied()
            .unwrap_or(false)
    }
}

/// Keyboard codes exposed to Forth programs. These match raylib's `KeyboardKey` values.
pub const TRACKED_KEYS: &[i32] = &[
    4, 5, 24, 25, 32, 39, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 59, 61, 65, 66,
    67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90,
    91, 92, 93, 96, 256, 257, 258, 259, 260, 261, 262, 263, 264, 265, 266, 267, 268, 269, 280, 281,
    282, 283, 284, 290, 291, 292, 293, 294, 295, 296, 297, 298, 299, 300, 301, 320, 321, 322, 323,
    324, 325, 326, 327, 328, 329, 330, 331, 332, 333, 334, 335, 336, 340, 341, 342, 343, 344, 345,
    346, 347, 348,
];
