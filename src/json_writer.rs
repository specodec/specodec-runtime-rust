pub struct JsonWriter {
    buf: Vec<u8>,
    first_item: Vec<bool>,
}

impl JsonWriter {
    pub fn new() -> Self {
        JsonWriter { buf: Vec::new(), first_item: Vec::new() }
    }

    fn escape(&mut self, s: &str) {
        for b in s.bytes() {
            match b {
                0x22 => { self.buf.extend_from_slice(b"\\\""); }
                0x5C => { self.buf.extend_from_slice(b"\\\\"); }
                0x08 => { self.buf.extend_from_slice(b"\\b"); }
                0x0C => { self.buf.extend_from_slice(b"\\f"); }
                0x0A => { self.buf.extend_from_slice(b"\\n"); }
                0x0D => { self.buf.extend_from_slice(b"\\r"); }
                0x09 => { self.buf.extend_from_slice(b"\\t"); }
                0x00..=0x1F => {
                    self.buf.extend_from_slice(b"\\u00");
                    let hex = format!("{:02x}", b);
                    self.buf.extend_from_slice(hex.as_bytes());
                }
                _ => { self.buf.push(b); }
            }
        }
    }

    fn b64_encode(data: &[u8]) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut s = String::new();
        for chunk in data.chunks(3) {
            let b0 = chunk[0];
            let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
            let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };
            s.push(CHARS[(b0 >> 2) as usize] as char);
            s.push(CHARS[(((b0 & 3) << 4) | (b1 >> 4)) as usize] as char);
            s.push(if chunk.len() > 1 { CHARS[(((b1 & 0xF) << 2) | (b2 >> 6)) as usize] as char } else { '=' });
            s.push(if chunk.len() > 2 { CHARS[(b2 & 0x3F) as usize] as char } else { '=' });
        }
        s
    }

    pub fn write_string(&mut self, value: &str) {
        self.buf.push(b'"');
        self.escape(value);
        self.buf.push(b'"');
    }

    pub fn write_bool(&mut self, value: bool) {
        if value { self.buf.extend_from_slice(b"true"); }
        else { self.buf.extend_from_slice(b"false"); }
    }

    pub fn write_int32(&mut self, value: i32) {
        let s = format!("{}", value);
        self.buf.extend_from_slice(s.as_bytes());
    }

    pub fn write_int64(&mut self, value: i64) {
        self.buf.push(b'"');
        let s = format!("{}", value);
        self.buf.extend_from_slice(s.as_bytes());
        self.buf.push(b'"');
    }

    pub fn write_uint32(&mut self, value: u32) {
        let s = format!("{}", value);
        self.buf.extend_from_slice(s.as_bytes());
    }

    pub fn write_uint64(&mut self, value: u64) {
        self.buf.push(b'"');
        let s = format!("{}", value);
        self.buf.extend_from_slice(s.as_bytes());
        self.buf.push(b'"');
    }

    fn fmt_float(&self, value: f64) -> String {
        let mut s = format!("{}", value);
        if s.ends_with(".0") {
            s.truncate(s.len() - 2);
        }
        s
    }

    pub fn write_float32(&mut self, value: f32) {
        if !value.is_finite() {
            panic!("float32: NaN/Infinity not valid JSON");
        }
        let s = self.fmt_float(value as f64);
        self.buf.extend_from_slice(s.as_bytes());
    }

    pub fn write_float64(&mut self, value: f64) {
        if !value.is_finite() {
            panic!("float64: NaN/Infinity not valid JSON");
        }
        let s = self.fmt_float(value);
        self.buf.extend_from_slice(s.as_bytes());
    }

    pub fn write_null(&mut self) {
        self.buf.extend_from_slice(b"null");
    }

    pub fn write_bytes(&mut self, value: &[u8]) {
        self.buf.push(b'"');
        let encoded = Self::b64_encode(value);
        self.buf.extend_from_slice(encoded.as_bytes());
        self.buf.push(b'"');
    }

    pub fn write_enum(&mut self, value: &str) {
        self.write_string(value);
    }

    pub fn begin_object(&mut self, _field_count: usize) {
        self.buf.push(b'{');
        self.first_item.push(true);
    }

    pub fn write_field(&mut self, name: &str) {
        let top = self.first_item.len() - 1;
        if !self.first_item[top] { self.buf.push(b','); }
        self.first_item[top] = false;
        self.buf.push(b'"');
        self.escape(name);
        self.buf.push(b'"');
        self.buf.push(b':');
    }

    pub fn end_object(&mut self) {
        self.first_item.pop();
        self.buf.push(b'}');
    }

    pub fn begin_array(&mut self, _element_count: usize) {
        self.buf.push(b'[');
        self.first_item.push(true);
    }

    pub fn next_element(&mut self) {
        let top = self.first_item.len() - 1;
        if !self.first_item[top] { self.buf.push(b','); }
        self.first_item[top] = false;
    }

    pub fn end_array(&mut self) {
        self.first_item.pop();
        self.buf.push(b']');
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.buf.clone()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }
}
