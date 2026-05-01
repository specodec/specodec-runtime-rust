use crate::json_reader::SCodecError;

pub struct GronWriter {
    lines: Vec<String>,
    segments: Vec<String>,
    nesting: Vec<NestInfo>,
}

struct NestInfo {
    depth: usize,
    array_index: i32,
}

impl GronWriter {
    pub fn new() -> Self {
        GronWriter {
            lines: Vec::new(),
            segments: vec!["json".to_string()],
            nesting: Vec::new(),
        }
    }

    fn build_path(&self) -> String {
        let mut r = self.segments[0].clone();
        for i in 1..self.segments.len() {
            if self.segments[i].starts_with('[') {
                r.push_str(&self.segments[i]);
            } else {
                r.push('.');
                r.push_str(&self.segments[i]);
            }
        }
        r
    }

    fn escape(s: &str) -> String {
        let mut r = String::new();
        for c in s.chars() {
            match c {
                '"' => r.push_str("\\\""),
                '\\' => r.push_str("\\\\"),
                '\u{08}' => r.push_str("\\b"),
                '\u{0C}' => r.push_str("\\f"),
                '\n' => r.push_str("\\n"),
                '\r' => r.push_str("\\r"),
                '\t' => r.push_str("\\t"),
                c if (c as u32) < 0x20 => {
                    r.push_str(&format!("\\u{:04x}", c as u32));
                }
                c => r.push(c),
            }
        }
        r
    }

    fn b64(data: &[u8]) -> String {
        const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut s = String::new();
        let mut i = 0;
        while i < data.len() {
            let b0 = data[i] as usize;
            let b1 = if i + 1 < data.len() { data[i + 1] as usize } else { 0 };
            let b2 = if i + 2 < data.len() { data[i + 2] as usize } else { 0 };
            s.push(CHARS[b0 >> 2] as char);
            s.push(CHARS[((b0 & 3) << 4) | (b1 >> 4)] as char);
            s.push(if i + 1 < data.len() { CHARS[((b1 & 0xF) << 2) | (b2 >> 6)] as char } else { '=' });
            s.push(if i + 2 < data.len() { CHARS[b2 & 0x3F] as char } else { '=' });
            i += 3;
        }
        s
    }

    fn emit(&mut self, raw: &str) {
        self.lines.push(format!("{} = {};", self.build_path(), raw));
    }

    pub fn write_string(&mut self, value: &str) {
        self.emit(&format!("\"{}\"", Self::escape(value)));
    }

    pub fn write_bool(&mut self, value: bool) {
        self.emit(if value { "true" } else { "false" });
    }

    pub fn write_int32(&mut self, value: i32) { self.emit(&value.to_string()); }
    pub fn write_int64(&mut self, value: i64) { self.emit(&format!("\"{}\"", value)); }
    pub fn write_uint32(&mut self, value: u32) { self.emit(&value.to_string()); }
    pub fn write_uint64(&mut self, value: u64) { self.emit(&format!("\"{}\"", value)); }

    pub fn write_float32(&mut self, value: f32) {
        if value.is_nan() || value.is_infinite() {
            panic!("NaN/Infinity");
        }
        if value == 0.0 && value.is_sign_negative() {
            self.emit("-0");
        } else {
            let mut r = format!("{}", value);
            if r.contains('.') && !r.contains('E') && !r.contains('e') {
                r = r.trim_end_matches('0').trim_end_matches('.').to_string();
                if r.is_empty() { r = "0".to_string(); }
            }
            self.emit(&r);
        }
    }

    pub fn write_float64(&mut self, value: f64) {
        if value.is_nan() || value.is_infinite() {
            panic!("NaN/Infinity");
        }
        if value == 0.0 && value.is_sign_negative() {
            self.emit("-0");
        } else {
            let mut r = format!("{}", value);
            if r.contains('.') && !r.contains('E') && !r.contains('e') {
                r = r.trim_end_matches('0').trim_end_matches('.').to_string();
                if r.is_empty() { r = "0".to_string(); }
            }
            self.emit(&r);
        }
    }

    pub fn write_null(&mut self) { self.emit("null"); }

    pub fn write_bytes(&mut self, value: &[u8]) {
        self.emit(&format!("\"{}\"", Self::b64(value)));
    }

    pub fn begin_object(&mut self, _field_count: usize) {
        self.lines.push(format!("{} = {{}};", self.build_path()));
        self.nesting.push(NestInfo { depth: self.segments.len(), array_index: -1 });
    }

    pub fn write_field(&mut self, name: &str) {
        let top = self.nesting.last().unwrap();
        if self.segments.len() > top.depth {
            *self.segments.last_mut().unwrap() = name.to_string();
        } else {
            self.segments.push(name.to_string());
        }
    }

    pub fn end_object(&mut self) {
        let info = self.nesting.pop().unwrap();
        self.segments.truncate(info.depth);
    }

    pub fn begin_array(&mut self, _element_count: usize) {
        self.lines.push(format!("{} = [];", self.build_path()));
        self.nesting.push(NestInfo { depth: self.segments.len(), array_index: -1 });
    }

    pub fn next_element(&mut self) {
        let info = self.nesting.last_mut().unwrap();
        info.array_index += 1;
        let seg = format!("[{}]", info.array_index);
        if self.segments.len() > info.depth {
            *self.segments.last_mut().unwrap() = seg;
        } else {
            self.segments.push(seg);
        }
    }

    pub fn end_array(&mut self) {
        let info = self.nesting.pop().unwrap();
        self.segments.truncate(info.depth);
    }

    pub fn write_enum(&mut self, value: &str) {
        let escaped = Self::escape(value);
        self.emit(&format!("\"{}\"", escaped));
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut r = self.lines.join("\n");
        r.push('\n');
        r.into_bytes()
    }
}
