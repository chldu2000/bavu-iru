use crate::crypto::strength::{self, StrengthResult};

#[tauri::command]
pub fn evaluate_password_strength(password: String) -> Result<StrengthResult, String> {
    Ok(strength::evaluate_strength(&password))
}
