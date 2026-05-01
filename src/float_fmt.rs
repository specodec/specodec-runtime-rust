// fmt_float32: returns the shortest decimal string that uniquely identifies
// the given f32 value and round-trips back to the same f32 bits.
//
// Rust's Display for f32 (format!("{}")) already uses the Ryu algorithm
// internally (via the `ryu` crate in std since Rust 1.x).
//
// TODO: if a standalone Ryu implementation is needed (e.g. for portability
// or to match other languages exactly), replace this body with an explicit
// Ryu f32 port.
pub fn fmt_float32(value: f32) -> String {
    let mut s = format!("{}", value);
    if s.ends_with(".0") {
        s.truncate(s.len() - 2);
    }
    s
}
