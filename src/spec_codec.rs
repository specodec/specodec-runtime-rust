use crate::spec_reader::SpecReader;
use crate::json_reader::{JsonReader, SCodecError};
use crate::msgpack_reader::MsgPackReader;

pub struct SpecCodec<T> {
    pub encode_json: fn(&T) -> Vec<u8>,
    pub encode_msgpack: fn(&T) -> Vec<u8>,
    pub decode: fn(&mut dyn SpecReader) -> Result<T, SCodecError>,
}

pub fn dispatch<T>(codec: &SpecCodec<T>, body: &[u8], content_type: &str) -> Result<T, SCodecError> {
    if content_type.contains("msgpack") {
        let mut r = MsgPackReader::new(body);
        (codec.decode)(&mut r)
    } else {
        let mut r = JsonReader::new(body)?;
        (codec.decode)(&mut r)
    }
}

pub fn respond<T>(codec: &SpecCodec<T>, obj: &T, accept: &str) -> Vec<u8> {
    if accept.contains("msgpack") {
        (codec.encode_msgpack)(obj)
    } else {
        (codec.encode_json)(obj)
    }
}
