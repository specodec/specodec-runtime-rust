use crate::json_reader::SCodecError;
pub struct GronReader<'a> { _data: &'a [u8] }
impl<'a> GronReader<'a> {
    pub fn new(data: &'a [u8]) -> Self { GronReader { _data: data } }
    pub fn read_string(&mut self) -> Result<String, SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn read_bool(&mut self) -> Result<bool, SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn read_int32(&mut self) -> Result<i32, SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn read_int64(&mut self) -> Result<i64, SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn read_uint32(&mut self) -> Result<u32, SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn read_uint64(&mut self) -> Result<u64, SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn read_float32(&mut self) -> Result<f32, SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn read_float64(&mut self) -> Result<f64, SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn read_null(&mut self) -> Result<(), SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn read_bytes(&mut self) -> Result<Vec<u8>, SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn begin_object(&mut self) -> Result<(), SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn has_next_field(&mut self) -> Result<bool, SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn read_field_name(&mut self) -> Result<String, SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn next_field_separator(&mut self) -> Result<(), SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn end_object(&mut self) -> Result<(), SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn begin_array(&mut self) -> Result<(), SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn has_next_element(&mut self) -> Result<bool, SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn next_element_separator(&mut self) -> Result<(), SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn end_array(&mut self) -> Result<(), SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn is_null(&mut self) -> Result<bool, SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
    pub fn skip(&mut self) -> Result<(), SCodecError> { Err(SCodecError::new("GronReader: not implemented")) }
}
