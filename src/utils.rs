// src/utils.rs

pub fn round_to_2_decimals(value: f32) -> f32 {
    // Use string formatting to get exactly 2 decimal places and then parse back
    // This avoids floating point precision issues
    format!("{:.2}", value).parse().unwrap_or(value)
}
