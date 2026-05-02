use crate::ryu::{float32_to_string, float64_to_string};

pub fn format_float32(value: f32) -> String {
    float32_to_string(value)
}

pub fn format_float64(value: f64) -> String {
    float64_to_string(value)
}