pub trait SpecWriter {
    fn write_string(&mut self, value: &str);
    fn write_bool(&mut self, value: bool);
    fn write_int32(&mut self, value: i32);
    fn write_int64(&mut self, value: i64);
    fn write_uint32(&mut self, value: u32);
    fn write_uint64(&mut self, value: u64);
    fn write_float32(&mut self, value: f32);
    fn write_float64(&mut self, value: f64);
    fn write_null(&mut self);
    fn write_bytes(&mut self, value: &[u8]);
    fn write_enum(&mut self, value: &str);
    fn begin_object(&mut self, field_count: usize);
    fn write_field(&mut self, name: &str);
    fn end_object(&mut self);
    fn begin_array(&mut self, element_count: usize);
    fn next_element(&mut self);
    fn end_array(&mut self);
    fn to_bytes(&self) -> Vec<u8>;
}
