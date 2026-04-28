pub struct GronWriter;
impl GronWriter {
    pub fn new() -> Self { GronWriter }
    pub fn write_string(&mut self, _value: &str) { unimplemented!("GronWriter: not implemented") }
    pub fn write_bool(&mut self, _value: bool) { unimplemented!("GronWriter: not implemented") }
    pub fn write_int32(&mut self, _value: i32) { unimplemented!("GronWriter: not implemented") }
    pub fn write_int64(&mut self, _value: i64) { unimplemented!("GronWriter: not implemented") }
    pub fn write_uint32(&mut self, _value: u32) { unimplemented!("GronWriter: not implemented") }
    pub fn write_uint64(&mut self, _value: u64) { unimplemented!("GronWriter: not implemented") }
    pub fn write_float32(&mut self, _value: f32) { unimplemented!("GronWriter: not implemented") }
    pub fn write_float64(&mut self, _value: f64) { unimplemented!("GronWriter: not implemented") }
    pub fn write_null(&mut self) { unimplemented!("GronWriter: not implemented") }
    pub fn write_bytes(&mut self, _value: &[u8]) { unimplemented!("GronWriter: not implemented") }
    pub fn begin_object(&mut self, _field_count: usize) { unimplemented!("GronWriter: not implemented") }
    pub fn write_field(&mut self, _name: &str) { unimplemented!("GronWriter: not implemented") }
    pub fn end_object(&mut self) { unimplemented!("GronWriter: not implemented") }
    pub fn begin_array(&mut self, _element_count: usize) { unimplemented!("GronWriter: not implemented") }
    pub fn next_element(&mut self) { unimplemented!("GronWriter: not implemented") }
    pub fn end_array(&mut self) { unimplemented!("GronWriter: not implemented") }
    pub fn to_bytes(&self) -> Vec<u8> { unimplemented!("GronWriter: not implemented") }
}
