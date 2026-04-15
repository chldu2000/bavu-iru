use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrengthResult {
    pub score: u8,      // 0-4
    pub label: String,   // 中文描述
    pub feedback: String, // 改进建议
}

const COMMON_PASSWORDS: &[&str] = &[
    "password",
    "123456",
    "12345678",
    "123456789",
    "1234567890",
    "qwerty",
    "abc123",
    "password1",
    "password123",
    "iloveyou",
    "admin",
    "welcome",
    "monkey",
    "dragon",
    "master",
    "login",
    "princess",
    "football",
    "shadow",
    "sunshine",
    "trustno1",
    "passw0rd",
    "hello",
    "charlie",
    "donald",
    "letmein",
    "qwerty123",
    "admin123",
    "root",
    "toor",
    "pass",
    "test",
    "guest",
    "master",
    "changeme",
    "123123",
    "654321",
    "superman",
    "qazwsx",
    "michael",
    "password2",
    "access",
    "flower",
    "hottie",
    "starwars",
    "batman",
    "whatever",
    "1234",
    "111111",
    "000000",
];

const LABELS: &[&str] = &["很弱", "弱", "一般", "强", "很强"];

const FEEDBACK_MESSAGES: &[&str] = &[
    "密码太弱，请使用更复杂的密码",
    "密码较弱，建议增加长度和复杂度",
    "密码强度一般，建议加入特殊字符",
    "密码强度较好，可以更加复杂",
    "密码强度很高，安全性良好",
];

pub fn evaluate_strength(password: &str) -> StrengthResult {
    // Empty password
    if password.is_empty() {
        return StrengthResult {
            score: 0,
            label: "空".to_string(),
            feedback: "请输入密码".to_string(),
        };
    }

    // Check common passwords (case-insensitive)
    let lower = password.to_lowercase();
    if COMMON_PASSWORDS.contains(&lower.as_str()) {
        return StrengthResult {
            score: 0,
            label: "很弱".to_string(),
            feedback: "这是常见密码，极易被破解".to_string(),
        };
    }

    let mut score: i8 = 0;

    // Length scoring
    let len = password.len();
    if len >= 8 {
        score += 1;
    }
    if len >= 12 {
        score += 1;
    }
    if len >= 16 {
        score += 1;
    }
    if len >= 20 {
        score += 1;
    }

    // Character diversity
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digits = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_ascii_alphanumeric());

    let diversity_count = [has_uppercase, has_lowercase, has_digits, has_special]
        .iter()
        .filter(|&&x| x)
        .count();

    score += diversity_count as i8;

    // Penalty: very short passwords
    if len < 6 {
        score -= 1;
    }
    if has_sequential_chars(password, 3) {
        score -= 1;
    }
    if has_repeated_chars(password, 3) {
        score -= 1;
    }

    // Clamp to 0-4
    let score = score.clamp(0, 4) as u8;

    StrengthResult {
        score,
        label: LABELS[score as usize].to_string(),
        feedback: FEEDBACK_MESSAGES[score as usize].to_string(),
    }
}

/// Detect sequential characters like "abc", "123", "cba", "321"
pub fn has_sequential_chars(password: &str, min_len: usize) -> bool {
    if min_len < 2 || password.len() < min_len {
        return false;
    }

    let chars: Vec<char> = password.chars().collect();

    for i in 0..=chars.len().saturating_sub(min_len) {
        // Check ascending sequence
        let mut asc_count = 1;
        for j in i + 1..chars.len() {
            if let Some(prev) = chars[j - 1].to_digit(36) {
                if let Some(curr) = chars[j].to_digit(36) {
                    if curr == prev + 1 {
                        asc_count += 1;
                        if asc_count >= min_len {
                            return true;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Check descending sequence
        let mut desc_count = 1;
        for j in i + 1..chars.len() {
            if let Some(prev) = chars[j - 1].to_digit(36) {
                if let Some(curr) = chars[j].to_digit(36) {
                    if curr + 1 == prev {
                        desc_count += 1;
                        if desc_count >= min_len {
                            return true;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    false
}

/// Detect repeated characters like "aaa", "111"
pub fn has_repeated_chars(password: &str, min_repeat: usize) -> bool {
    if min_repeat < 2 || password.is_empty() {
        return false;
    }

    let chars: Vec<char> = password.chars().collect();

    let mut count = 1;
    for i in 1..chars.len() {
        if chars[i] == chars[i - 1] {
            count += 1;
            if count >= min_repeat {
                return true;
            }
        } else {
            count = 1;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_password() {
        let result = evaluate_strength("");
        assert_eq!(result.score, 0);
        assert_eq!(result.label, "空");
        assert_eq!(result.feedback, "请输入密码");
    }

    #[test]
    fn test_common_password() {
        let result = evaluate_strength("password");
        assert_eq!(result.score, 0);
        assert!(result.feedback.contains("常见"), "Expected '常见' in feedback, got: {}", result.feedback);
    }

    #[test]
    fn test_short_password() {
        let result = evaluate_strength("ab12");
        assert!(result.score <= 1, "Expected score <= 1, got: {}", result.score);
    }

    #[test]
    fn test_strong_password() {
        let result = evaluate_strength("X#9kLm$2pQwR!nB7vF");
        assert!(result.score >= 3, "Expected score >= 3, got: {}", result.score);
    }

    #[test]
    fn test_medium_password() {
        let result = evaluate_strength("hello123");
        assert!((1..=3).contains(&result.score), "Expected score 1-3, got: {}", result.score);
    }

    #[test]
    fn test_sequential_detection() {
        assert!(has_sequential_chars("abc123", 3), "'abc123' should be sequential");
        assert!(!has_sequential_chars("a1b2c3", 3), "'a1b2c3' should not be sequential");
    }

    #[test]
    fn test_repeated_detection() {
        assert!(has_repeated_chars("aaabbb", 3), "'aaabbb' has repeated chars");
        assert!(!has_repeated_chars("ababab", 3), "'ababab' does not have repeated chars");
    }

    #[test]
    fn test_only_lowercase_short() {
        let result = evaluate_strength("hello");
        assert!(result.score <= 1, "Expected low score for 'hello', got: {}", result.score);
    }
}
