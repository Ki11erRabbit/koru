use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;
use koru_core::styled_text::ColorType;

static COLOR_DEFINITIONS: LazyLock<Mutex<ColorDefinitions>> = LazyLock::new(|| {
    Mutex::new(ColorDefinitions::new())
});

pub struct ColorDefinitions {
    map: HashMap<ColorType, (u8, u8, u8)>,
}

impl ColorDefinitions {
    fn new() -> ColorDefinitions {
        Self {
            map: HashMap::new(),
        }
    }
    
    fn insert_internal(&mut self, color: ColorType, value: (u8, u8, u8)) {
        self.map.insert(color, value);
    }
    
    /// Fetches the color or returns black if the color type isn't found
    fn get_internal(&self, color: &ColorType) -> (u8, u8, u8) {
        if let Some(value) = self.map.get(color) {
            *value
        } else {
            (0, 0, 0)
        }
    }
    
    pub fn insert(color: ColorType, value: (u8, u8, u8)) {
        let mut guard = COLOR_DEFINITIONS.lock().expect("Mutex poisoned");
        guard.insert_internal(color, value);
    }

    /// Fetches the color or returns black if the color type isn't found
    pub fn get(color: &ColorType) -> (u8, u8, u8) {
        let guard = COLOR_DEFINITIONS.lock().expect("Mutex poisoned");
        guard.get_internal(color)
    }
}