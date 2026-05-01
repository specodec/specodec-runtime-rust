use crate::spec_reader::SpecReader;
use crate::spec_writer::SpecWriter;
use crate::json_reader::{JsonReader, SCodecError};
use crate::json_writer::JsonWriter;
use crate::msgpack_reader::MsgPackReader;
use crate::msgpack_writer::MsgPackWriter;
use crate::gron_writer::GronWriter;
use crate::gron_reader::GronReader;

pub struct SpecCodec<T> {
    pub encode: fn(&T, &mut dyn SpecWriter),
    pub decode: fn(&mut dyn SpecReader) -> Result<T, SCodecError>,
}

// ---------------------------------------------------------------------------
// FormatEntry
// ---------------------------------------------------------------------------
pub struct FormatEntry {
    pub content_type: &'static str,
    pub new_writer: fn() -> Box<dyn SpecWriter>,
    pub new_reader: fn(&[u8]) -> Result<Box<dyn SpecReader>, SCodecError>,
}

// ---------------------------------------------------------------------------
// FormatRegistry
// ---------------------------------------------------------------------------
pub struct FormatRegistry {
    entries: Vec<FormatEntry>,
}

impl FormatRegistry {
    pub fn new() -> Self { FormatRegistry { entries: Vec::new() } }

    pub fn register(mut self, e: FormatEntry) -> Self {
        self.entries.push(e);
        self
    }

    pub fn match_format(&self, content_type: &str) -> &FormatEntry {
        for e in &self.entries {
            let sub = e.content_type.split('/').nth(1).unwrap_or("");
            if content_type.contains(sub) { return e; }
        }
        &self.entries[0]
    }
}

// ---------------------------------------------------------------------------
// Default registry
// ---------------------------------------------------------------------------
pub fn default_registry() -> FormatRegistry {
    FormatRegistry::new()
        .register(FormatEntry {
            content_type: "application/json",
            new_writer: || Box::new(JsonWriter::new()),
            new_reader: |body| Ok(Box::new(JsonReader::new(body)?) as Box<dyn SpecReader>),
        })
        .register(FormatEntry {
            content_type: "application/msgpack",
            new_writer: || Box::new(MsgPackWriter::new()),
            new_reader: |body| Ok(Box::new(MsgPackReader::new(body)) as Box<dyn SpecReader>),
        })
        .register(FormatEntry {
            content_type: "application/gron",
            new_writer: || Box::new(GronWriter::new()),
            new_reader: |body| Ok(Box::new(GronReader::new(body)) as Box<dyn SpecReader>),
        })
}

// ---------------------------------------------------------------------------
// dispatch / respond
// ---------------------------------------------------------------------------
pub fn dispatch<T>(codec: &SpecCodec<T>, body: &[u8], content_type: &str) -> Result<T, SCodecError> {
    let reg = default_registry();
    let fmt = reg.match_format(content_type);
    let mut r = (fmt.new_reader)(body)?;
    
    (codec.decode)(r.as_mut())
}

pub struct RespondResult {
    pub body: Vec<u8>,
    pub content_type: String,
}

pub fn respond<T>(codec: &SpecCodec<T>, obj: &T, accept: &str) -> RespondResult {
    let reg = default_registry();
    let fmt = reg.match_format(accept);
    let mut w = (fmt.new_writer)();
    (codec.encode)(obj, w.as_mut());
    RespondResult {
        body: w.to_bytes(),
        content_type: fmt.content_type.to_string(),
    }
}
