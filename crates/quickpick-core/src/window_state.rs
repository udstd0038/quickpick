use crate::repository::{AppPaths, atomic_write};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs};
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
pub type WindowStates = BTreeMap<String, WindowGeometry>;
pub fn load(paths: &AppPaths) -> WindowStates {
    fs::read(paths.config_dir.join("native-window-state.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<WindowStates>(&bytes).ok())
        .unwrap_or_default()
        .into_iter()
        .filter(|(role, rect)| {
            matches!(role.as_str(), "selection" | "result" | "input")
                && rect.width >= 100
                && rect.height >= 32
                && rect.width <= 16384
                && rect.height <= 16384
        })
        .collect()
}
pub fn save(paths: &AppPaths, states: &WindowStates) -> Result<(), String> {
    let bytes = serde_json::to_vec(states).map_err(|_| "window state".to_string())?;
    atomic_write(&paths.config_dir.join("native-window-state.json"), &bytes)
        .map_err(|_| "window state".to_string())
}
