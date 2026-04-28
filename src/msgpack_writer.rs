pub struct MsgPackWriter {
    buf: Vec<u8>,
}

impl MsgPackWriter {
    pub fn new() -> Self { MsgPackWriter { buf: Vec::with_capacity(256) } }

    fn write_byte(&mut self, b: u8) { self.buf.push(b); }
    fn write_u16(&mut self, v: u16) { self.buf.extend_from_slice(&v.to_be_bytes()); }
    fn write_u32(&mut self, v: u32) { self.buf.extend_from_slice(&v.to_be_bytes()); }
    fn write_u64(&mut self, v: u64) { self.buf.extend_from_slice(&v.to_be_bytes()); }

    pub fn write_string(&mut self, value: &str) {
        let bytes = value.as_bytes();
        let len = bytes.len();
        if len <= 0x1F { self.write_byte(0xA0 | len as u8); }
        else if len <= 0xFF { self.write_byte(0xD9); self.write_byte(len as u8); }
        else if len <= 0xFFFF { self.write_byte(0xDA); self.write_u16(len as u16); }
        else { self.write_byte(0xDB); self.write_u32(len as u32); }
        self.buf.extend_from_slice(bytes);
    }

    pub fn write_bool(&mut self, value: bool) { self.write_byte(if value { 0xC3 } else { 0xC2 }); }

    pub fn write_int32(&mut self, value: i32) {
        if value >= 0 && value <= 0x7F { self.write_byte(value as u8); }
        else if value < 0 && value >= -0x20 { self.write_byte(value as u8); }
        else if value >= 0 && value <= 0xFF { self.write_byte(0xCC); self.write_byte(value as u8); }
        else if value >= 0 && value <= 0xFFFF { self.write_byte(0xCD); self.write_u16(value as u16); }
        else if value >= 0 { self.write_byte(0xCE); self.write_u32(value as u32); }
        else if value >= -0x80 { self.write_byte(0xD0); self.write_byte(value as u8); }
        else if value >= -0x8000 { self.write_byte(0xD1); self.write_u16(value as u16); }
        else { self.write_byte(0xD2); self.write_u32(value as u32); }
    }

    pub fn write_int64(&mut self, value: i64) {
        if value >= 0 && value <= 0x7F { self.write_byte(value as u8); }
        else if value < 0 && value >= -0x20 { self.write_byte(value as u8); }
        else if value >= 0 && value <= 0xFF { self.write_byte(0xCC); self.write_byte(value as u8); }
        else if value >= 0 && value <= 0xFFFF { self.write_byte(0xCD); self.write_u16(value as u16); }
        else if value >= 0 && value <= (u32::MAX as i64) { self.write_byte(0xCE); self.write_u32(value as u32); }
        else if value >= 0 { self.write_byte(0xCF); self.write_u64(value as u64); }
        else if value >= -0x80 { self.write_byte(0xD0); self.write_byte(value as u8); }
        else if value >= -0x8000 { self.write_byte(0xD1); self.write_u16(value as u16); }
        else if value >= -0x80000000 { self.write_byte(0xD2); self.write_u32(value as u32); }
        else { self.write_byte(0xD3); self.write_u64(value as u64); }
    }

    pub fn write_uint32(&mut self, value: u32) {
        if value <= 0x7F { self.write_byte(value as u8); }
        else if value <= 0xFF { self.write_byte(0xCC); self.write_byte(value as u8); }
        else if value <= 0xFFFF { self.write_byte(0xCD); self.write_u16(value as u16); }
        else { self.write_byte(0xCE); self.write_u32(value); }
    }

    pub fn write_uint64(&mut self, value: u64) {
        if value <= 0x7F { self.write_byte(value as u8); }
        else if value <= 0xFF { self.write_byte(0xCC); self.write_byte(value as u8); }
        else if value <= 0xFFFF { self.write_byte(0xCD); self.write_u16(value as u16); }
        else if value <= 0xFFFFFFFF { self.write_byte(0xCE); self.write_u32(value as u32); }
        else { self.write_byte(0xCF); self.write_u64(value); }
    }

    pub fn write_float32(&mut self, value: f32) {
        self.write_byte(0xCA);
        self.buf.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_float64(&mut self, value: f64) {
        self.write_byte(0xCB);
        self.buf.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_null(&mut self) { self.write_byte(0xC0); }

    pub fn write_bytes(&mut self, value: &[u8]) {
        let len = value.len();
        if len <= 0xFF { self.write_byte(0xC4); self.write_byte(len as u8); }
        else if len <= 0xFFFF { self.write_byte(0xC5); self.write_u16(len as u16); }
        else { self.write_byte(0xC6); self.write_u32(len as u32); }
        self.buf.extend_from_slice(value);
    }

    pub fn begin_object(&mut self, field_count: usize) {
        if field_count <= 0x0F { self.write_byte(0x80 | field_count as u8); }
        else if field_count <= 0xFFFF { self.write_byte(0xDE); self.write_u16(field_count as u16); }
        else { self.write_byte(0xDF); self.write_u32(field_count as u32); }
    }

    pub fn write_field(&mut self, name: &str) { self.write_string(name); }
    pub fn end_object(&mut self) {}

    pub fn begin_array(&mut self, element_count: usize) {
        if element_count <= 0x0F { self.write_byte(0x90 | element_count as u8); }
        else if element_count <= 0xFFFF { self.write_byte(0xDC); self.write_u16(element_count as u16); }
        else { self.write_byte(0xDD); self.write_u32(element_count as u32); }
    }

    pub fn next_element(&mut self) {}
    pub fn end_array(&mut self) {}

    pub fn to_bytes(&self) -> Vec<u8> { self.buf.clone() }
    pub fn into_bytes(self) -> Vec<u8> { self.buf }
}
