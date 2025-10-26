use std::fs;
use crate::models::WhiteboardState;

pub fn save_to_file(state: &WhiteboardState, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(state)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn load_from_file(path: &str) -> Result<WhiteboardState, Box<dyn std::error::Error>> {
    let json = fs::read_to_string(path)?;
    let state: WhiteboardState = serde_json::from_str(&json)?;
    Ok(state)
}
