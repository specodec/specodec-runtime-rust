// Implement SpecWriter trait for JsonWriter, MsgPackWriter, GronWriter
use crate::spec_writer::SpecWriter;
use crate::json_writer::JsonWriter;
use crate::msgpack_writer::MsgPackWriter;
use crate::gron_writer::GronWriter;

impl SpecWriter for JsonWriter {
    fn write_string(&mut self, v: &str) { self.write_string(v); }
    fn write_bool(&mut self, v: bool) { self.write_bool(v); }
    fn write_int32(&mut self, v: i32) { self.write_int32(v); }
    fn write_int64(&mut self, v: i64) { self.write_int64(v); }
    fn write_uint32(&mut self, v: u32) { self.write_uint32(v); }
    fn write_uint64(&mut self, v: u64) { self.write_uint64(v); }
    fn write_float32(&mut self, v: f32) { self.write_float32(v); }
    fn write_float64(&mut self, v: f64) { self.write_float64(v); }
    fn write_null(&mut self) { self.write_null(); }
    fn write_bytes(&mut self, v: &[u8]) { self.write_bytes(v); }
    fn write_enum(&mut self, v: &str) { self.write_enum(v); }
    fn begin_object(&mut self, n: usize) { self.begin_object(n); }
    fn write_field(&mut self, name: &str) { self.write_field(name); }
    fn end_object(&mut self) { self.end_object(); }
    fn begin_array(&mut self, n: usize) { self.begin_array(n); }
    fn next_element(&mut self) { self.next_element(); }
    fn end_array(&mut self) { self.end_array(); }
    fn to_bytes(&self) -> Vec<u8> { self.to_bytes() }
}

impl SpecWriter for MsgPackWriter {
    fn write_string(&mut self, v: &str) { self.write_string(v); }
    fn write_bool(&mut self, v: bool) { self.write_bool(v); }
    fn write_int32(&mut self, v: i32) { self.write_int32(v); }
    fn write_int64(&mut self, v: i64) { self.write_int64(v); }
    fn write_uint32(&mut self, v: u32) { self.write_uint32(v); }
    fn write_uint64(&mut self, v: u64) { self.write_uint64(v); }
    fn write_float32(&mut self, v: f32) { self.write_float32(v); }
    fn write_float64(&mut self, v: f64) { self.write_float64(v); }
    fn write_null(&mut self) { self.write_null(); }
    fn write_bytes(&mut self, v: &[u8]) { self.write_bytes(v); }
    fn write_enum(&mut self, v: &str) { self.write_string(v); }
    fn begin_object(&mut self, n: usize) { self.begin_object(n); }
    fn write_field(&mut self, name: &str) { self.write_field(name); }
    fn end_object(&mut self) { self.end_object(); }
    fn begin_array(&mut self, n: usize) { self.begin_array(n); }
    fn next_element(&mut self) { self.next_element(); }
    fn end_array(&mut self) { self.end_array(); }
    fn to_bytes(&self) -> Vec<u8> { self.to_bytes() }
}

impl SpecWriter for GronWriter {
    fn write_string(&mut self, v: &str) { self.write_string(v); }
    fn write_bool(&mut self, v: bool) { self.write_bool(v); }
    fn write_int32(&mut self, v: i32) { self.write_int32(v); }
    fn write_int64(&mut self, v: i64) { self.write_int64(v); }
    fn write_uint32(&mut self, v: u32) { self.write_uint32(v); }
    fn write_uint64(&mut self, v: u64) { self.write_uint64(v); }
    fn write_float32(&mut self, v: f32) { self.write_float32(v); }
    fn write_float64(&mut self, v: f64) { self.write_float64(v); }
    fn write_null(&mut self) { self.write_null(); }
    fn write_bytes(&mut self, v: &[u8]) { self.write_bytes(v); }
    fn write_enum(&mut self, v: &str) { self.write_enum(v); }
    fn begin_object(&mut self, n: usize) { self.begin_object(n); }
    fn write_field(&mut self, name: &str) { self.write_field(name); }
    fn end_object(&mut self) { self.end_object(); }
    fn begin_array(&mut self, n: usize) { self.begin_array(n); }
    fn next_element(&mut self) { self.next_element(); }
    fn end_array(&mut self) { self.end_array(); }
    fn to_bytes(&self) -> Vec<u8> { self.to_bytes() }
}
