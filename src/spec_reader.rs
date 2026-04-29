use crate::json_reader::SCodecError;

pub trait SpecReader {
    fn begin_object(&mut self) -> Result<(), SCodecError>;
    fn has_next_field(&mut self) -> Result<bool, SCodecError>;
    fn read_field_name(&mut self) -> Result<String, SCodecError>;
    fn end_object(&mut self) -> Result<(), SCodecError>;
    fn begin_array(&mut self) -> Result<(), SCodecError>;
    fn has_next_element(&mut self) -> Result<bool, SCodecError>;
    fn end_array(&mut self) -> Result<(), SCodecError>;
    fn read_string(&mut self) -> Result<String, SCodecError>;
    fn read_bool(&mut self) -> Result<bool, SCodecError>;
    fn read_int32(&mut self) -> Result<i32, SCodecError>;
    fn read_int64(&mut self) -> Result<i64, SCodecError>;
    fn read_uint32(&mut self) -> Result<u32, SCodecError>;
    fn read_uint64(&mut self) -> Result<u64, SCodecError>;
    fn read_float32(&mut self) -> Result<f32, SCodecError>;
    fn read_float64(&mut self) -> Result<f64, SCodecError>;
    fn read_null(&mut self) -> Result<(), SCodecError>;
    fn read_bytes(&mut self) -> Result<Vec<u8>, SCodecError>;
    fn read_enum(&mut self) -> Result<String, SCodecError>;
    fn is_null(&mut self) -> Result<bool, SCodecError>;
    fn skip(&mut self) -> Result<(), SCodecError>;
}
