use rand::seq::SliceRandom;
use rand::RngExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorOptions {
    pub length: usize,
    pub uppercase: bool,
    pub lowercase: bool,
    pub digits: bool,
    pub special: bool,
    pub exclude_chars: String,
}

impl Default for GeneratorOptions {
    fn default() -> Self {
        Self {
            length: 20,
            uppercase: true,
            lowercase: true,
            digits: true,
            special: true,
            exclude_chars: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedPassword {
    pub password: String,
}

const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SPECIAL: &str = "!@#$%^&*()_+-=[]{}|;:,,.>?";

#[tauri::command]
pub fn generate_password(options: GeneratorOptions) -> Result<GeneratedPassword, String> {
    if options.length < 4 || options.length > 128 {
        return Err("Length must be between 4 and 128".to_string());
    }

    let exclude: Vec<char> = options.exclude_chars.chars().collect();
    let mut rng = rand::rng();

    let mut pool = String::new();
    let mut required_chars: Vec<char> = Vec::new();

    if options.uppercase {
        pool.push_str(UPPERCASE);
        let available: Vec<char> = UPPERCASE.chars().filter(|c| !exclude.contains(c)).collect();
        if available.is_empty() {
            return Err("All uppercase characters are excluded".to_string());
        }
        let ch = available[rng.random_range(..available.len())];
        required_chars.push(ch);
    }
    if options.lowercase {
        pool.push_str(LOWERCASE);
        let available: Vec<char> = LOWERCASE.chars().filter(|c| !exclude.contains(c)).collect();
        if available.is_empty() {
            return Err("All lowercase characters are excluded".to_string());
        }
        let ch = available[rng.random_range(..available.len())];
        required_chars.push(ch);
    }
    if options.digits {
        pool.push_str(DIGITS);
        let available: Vec<char> = DIGITS.chars().filter(|c| !exclude.contains(c)).collect();
        if available.is_empty() {
            return Err("All digit characters are excluded".to_string());
        }
        let ch = available[rng.random_range(..available.len())];
        required_chars.push(ch);
    }
    if options.special {
        pool.push_str(SPECIAL);
        let available: Vec<char> = SPECIAL.chars().filter(|c| !exclude.contains(c)).collect();
        if available.is_empty() {
            return Err("All special characters are excluded".to_string());
        }
        let ch = available[rng.random_range(..available.len())];
        required_chars.push(ch);
    }

    if pool.is_empty() {
        return Err("No character types selected".to_string());
    }

    // Filter pool by excluded chars
    let pool_chars: Vec<char> = pool.chars().filter(|c| !exclude.contains(c)).collect();
    if pool_chars.is_empty() {
        return Err("Character pool is empty after exclusions".to_string());
    }

    // Ensure length can accommodate all required chars
    if required_chars.len() > options.length {
        return Err("Length too short for all enabled character types".to_string());
    }

    // Fill remaining slots from pool
    let remaining = options.length - required_chars.len();
    let mut result = required_chars;
    for _ in 0..remaining {
        let ch = pool_chars[rng.random_range(..pool_chars.len())];
        result.push(ch);
    }

    // Shuffle to randomize guaranteed char positions
    result.shuffle(&mut rng);

    Ok(GeneratedPassword {
        password: result.into_iter().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options_generates_password() {
        let options = GeneratorOptions::default();
        let result = generate_password(options).unwrap();
        assert_eq!(result.password.len(), 20);
    }

    #[test]
    fn test_custom_length() {
        let options = GeneratorOptions {
            length: 32,
            ..Default::default()
        };
        let result = generate_password(options).unwrap();
        assert_eq!(result.password.len(), 32);
    }

    #[test]
    fn test_only_digits() {
        let options = GeneratorOptions {
            length: 20,
            uppercase: false,
            lowercase: false,
            digits: true,
            special: false,
            exclude_chars: String::new(),
        };
        let result = generate_password(options).unwrap();
        assert!(result.password.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_exclude_chars() {
        let excluded = "abcXYZ012";
        let options = GeneratorOptions {
            length: 50,
            exclude_chars: excluded.to_string(),
            ..Default::default()
        };
        let result = generate_password(options).unwrap();
        for c in result.password.chars() {
            assert!(!excluded.contains(c), "Excluded char '{}' found in password", c);
        }
    }

    #[test]
    fn test_no_char_types_fails() {
        let options = GeneratorOptions {
            length: 20,
            uppercase: false,
            lowercase: false,
            digits: false,
            special: false,
            exclude_chars: String::new(),
        };
        let result = generate_password(options);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_length_fails() {
        let options = GeneratorOptions {
            length: 2,
            ..Default::default()
        };
        let result = generate_password(options);
        assert!(result.is_err());
    }

    #[test]
    fn test_all_enabled_types_present() {
        let options = GeneratorOptions {
            length: 50,
            ..Default::default()
        };
        let result = generate_password(options).unwrap();
        let pwd = &result.password;
        assert!(pwd.chars().any(|c| UPPERCASE.contains(c)), "No uppercase found");
        assert!(pwd.chars().any(|c| LOWERCASE.contains(c)), "No lowercase found");
        assert!(pwd.chars().any(|c| DIGITS.contains(c)), "No digits found");
        assert!(pwd.chars().any(|c| SPECIAL.contains(c)), "No special chars found");
    }
}
