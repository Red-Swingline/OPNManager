use std::sync::Mutex;

pub struct PinCache {
    pin: Mutex<Option<String>>,
}

impl PinCache {
    pub fn new() -> Self {
        PinCache {
            pin: Mutex::new(None),
        }
    }

    pub fn set_pin(&self, pin: String) {
        let mut cache = self.pin.lock().unwrap();
        *cache = Some(pin);
    }

    pub fn get_pin(&self) -> Option<String> {
        let cache = self.pin.lock().unwrap();
        cache.clone()
    }

    pub fn clear_pin(&self) {
        let mut cache = self.pin.lock().unwrap();
        *cache = None;
    }
}

#[tauri::command]
pub fn set_pin(pin: String, pin_cache: tauri::State<'_, PinCache>) {
    pin_cache.set_pin(pin);
}

#[tauri::command]
pub fn clear_pin(pin_cache: tauri::State<'_, PinCache>) {
    pin_cache.clear_pin();
}

#[tauri::command]
pub fn verify_pin(
    pin: String,
    database: tauri::State<'_, crate::db::Database>,
    pin_cache: tauri::State<'_, PinCache>,
) -> Result<bool, String> {
    let is_valid = database
        .verify_pin(&pin)
        .map_err(|e| format!("Failed to verify PIN: {}", e))?;

    if is_valid {
        pin_cache.set_pin(pin);
    }

    Ok(is_valid)
}
