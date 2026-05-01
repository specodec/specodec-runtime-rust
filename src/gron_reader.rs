use crate::json_reader::SCodecError;
use crate::spec_reader::SpecReader;

pub struct GronReader {
    lines: Vec<(String, String)>,
    cursor: usize,
    ctx: Vec<CtxInfo>,
}

struct CtxInfo {
    prefix: String,
    #[allow(dead_code)]
    ctx_type: String,
    index: i32,
}

impl GronReader {
    pub fn new(data: &[u8]) -> Self {
        let text = String::from_utf8_lossy(data);
        let mut lines = Vec::new();
        for raw in text.split('\n') {
            let line = raw.trim();
            if line.is_empty() { continue; }
            let eq = match line.find(" = ") {
                Some(i) => i,
                None => continue,
            };
            let path = line[..eq].to_string();
            let mut val = line[eq + 3..].to_string();
            if val.ends_with(';') { val.pop(); }
            lines.push((path, val));
        }
        GronReader { lines, cursor: 0, ctx: Vec::new() }
    }

    fn unescape(s: &str) -> Result<String, SCodecError> {
        if s.len() < 2 || !s.starts_with('"') || !s.ends_with('"') {
            return Err(SCodecError::new("gron: expected quoted string"));
        }
        let chars: Vec<char> = s[1..s.len()-1].chars().collect();
        let mut r = String::new();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '\\' && i + 1 < chars.len() {
                i += 1;
                match chars[i] {
                    '"' => r.push('"'),
                    '\\' => r.push('\\'),
                    '/' => r.push('/'),
                    'b' => r.push('\u{08}'),
                    'f' => r.push('\u{0C}'),
                    'n' => r.push('\n'),
                    'r' => r.push('\r'),
                    't' => r.push('\t'),
                    'u' => {
                        let hex: String = chars[i+1..i+5.min(chars.len())].iter().collect();
                        if let Ok(v) = u16::from_str_radix(&hex, 16) {
                            if let Some(c) = char::from_u32(v as u32) { r.push(c); }
                        }
                        i += 4;
                    }
                    c => r.push(c),
                }
            } else {
                r.push(chars[i]);
            }
            i += 1;
        }
        Ok(r)
    }

    fn b64decode(s: &str) -> Result<Vec<u8>, SCodecError> {
        const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = Vec::new();
        let bytes = s.as_bytes();
        let pad_count = bytes.iter().rev().take_while(|&&b| b == b'=').count();
        let mut i = 0;
        while i < bytes.len() && bytes[i] != b'=' {
            let b0 = CHARS.iter().position(|&c| c == bytes[i]).unwrap_or(0) as u32; i += 1;
            let b1 = if i < bytes.len() && bytes[i] != b'=' { let v = CHARS.iter().position(|&c| c == bytes[i]).unwrap_or(0) as u32; i += 1; v } else { 0 };
            let b2 = if i < bytes.len() && bytes[i] != b'=' { let v = CHARS.iter().position(|&c| c == bytes[i]).unwrap_or(0) as u32; i += 1; v } else { 0 };
            let b3 = if i < bytes.len() && bytes[i] != b'=' { let v = CHARS.iter().position(|&c| c == bytes[i]).unwrap_or(0) as u32; i += 1; v } else { 0 };
            result.push(((b0 << 2) | (b1 >> 4)) as u8);
            result.push((((b1 & 0xF) << 4) | (b2 >> 2)) as u8);
            result.push((((b2 & 3) << 6) | b3) as u8);
        }
        result.truncate(result.len().saturating_sub(pad_count));
        Ok(result)
    }

    pub fn read_string(&mut self) -> Result<String, SCodecError> { let v = Self::unescape(&self.lines[self.cursor].1)?; self.cursor += 1; Ok(v) }
    pub fn read_bool(&mut self) -> Result<bool, SCodecError> { let v = self.lines[self.cursor].1 == "true"; self.cursor += 1; Ok(v) }
    pub fn read_int32(&mut self) -> Result<i32, SCodecError> { let v = self.lines[self.cursor].1.parse::<i32>().map_err(|_| SCodecError::new("gron: invalid int32"))?; self.cursor += 1; Ok(v) }
    pub fn read_int64(&mut self) -> Result<i64, SCodecError> { let v = Self::unescape(&self.lines[self.cursor].1)?.parse::<i64>().map_err(|_| SCodecError::new("gron: invalid int64"))?; self.cursor += 1; Ok(v) }
    pub fn read_uint32(&mut self) -> Result<u32, SCodecError> { let v = self.lines[self.cursor].1.parse::<u32>().map_err(|_| SCodecError::new("gron: invalid uint32"))?; self.cursor += 1; Ok(v) }
    pub fn read_uint64(&mut self) -> Result<u64, SCodecError> { let v = Self::unescape(&self.lines[self.cursor].1)?.parse::<u64>().map_err(|_| SCodecError::new("gron: invalid uint64"))?; self.cursor += 1; Ok(v) }
    pub fn read_float32(&mut self) -> Result<f32, SCodecError> {
        let v = &self.lines[self.cursor].1; self.cursor += 1;
        if v == "-0" { return Ok(-0.0); }
        v.parse::<f32>().map_err(|_| SCodecError::new("gron: invalid float32"))
    }
    pub fn read_float64(&mut self) -> Result<f64, SCodecError> {
        let v = &self.lines[self.cursor].1; self.cursor += 1;
        if v == "-0" { return Ok(-0.0); }
        v.parse::<f64>().map_err(|_| SCodecError::new("gron: invalid float64"))
    }
    pub fn read_null(&mut self) -> Result<(), SCodecError> {
        if self.lines[self.cursor].1 != "null" { return Err(SCodecError::new("gron: expected null")); }
        self.cursor += 1; Ok(())
    }
    pub fn read_bytes(&mut self) -> Result<Vec<u8>, SCodecError> {
        let s = Self::unescape(&self.lines[self.cursor].1)?;
        self.cursor += 1;
        Self::b64decode(&s)
    }

    pub fn begin_object(&mut self) -> Result<(), SCodecError> {
        let line = &self.lines[self.cursor]; self.cursor += 1;
        self.ctx.push(CtxInfo { prefix: line.0.clone(), ctx_type: "object".to_string(), index: -1 });
        Ok(())
    }

    pub fn has_next_field(&mut self) -> Result<bool, SCodecError> {
        if self.cursor >= self.lines.len() { return Ok(false); }
        let pfx = format!("{}.", self.ctx.last().unwrap().prefix);
        let p = &self.lines[self.cursor].0;
        if !p.starts_with(&pfx) { return Ok(false); }
        let rem = &p[pfx.len()..];
        Ok(!rem.contains('.') && !rem.contains('['))
    }

    pub fn read_field_name(&mut self) -> Result<String, SCodecError> {
        let pfx = format!("{}.", self.ctx.last().unwrap().prefix);
        Ok(self.lines[self.cursor].0[pfx.len()..].to_string())
    }

    pub fn end_object(&mut self) -> Result<(), SCodecError> { self.ctx.pop(); Ok(()) }

    pub fn begin_array(&mut self) -> Result<(), SCodecError> {
        let line = &self.lines[self.cursor]; self.cursor += 1;
        self.ctx.push(CtxInfo { prefix: line.0.clone(), ctx_type: "array".to_string(), index: -1 });
        Ok(())
    }

    pub fn has_next_element(&mut self) -> Result<bool, SCodecError> {
        if self.cursor >= self.lines.len() { return Ok(false); }
        let arr = self.ctx.last().unwrap();
        let ni = arr.index + 1;
        let exp = format!("{}[{}]", arr.prefix, ni);
        let p = &self.lines[self.cursor].0;
        Ok(p == &exp || p.starts_with(&format!("{}.", exp)) || p.starts_with(&format!("{}[", exp)))
    }

    pub fn next_element(&mut self) -> Result<(), SCodecError> { self.ctx.last_mut().unwrap().index += 1; Ok(()) }
    pub fn end_array(&mut self) -> Result<(), SCodecError> { self.ctx.pop(); Ok(()) }

    pub fn is_null(&mut self) -> Result<bool, SCodecError> {
        Ok(self.cursor < self.lines.len() && self.lines[self.cursor].1 == "null")
    }

    pub fn skip(&mut self) -> Result<(), SCodecError> {
        let sp = self.lines[self.cursor].0.clone(); self.cursor += 1;
        while self.cursor < self.lines.len() {
            let np = &self.lines[self.cursor].0;
            if np.len() > sp.len() && (np.starts_with(&format!("{}.", sp)) || np.starts_with(&format!("{}[", sp))) {
                self.cursor += 1;
            } else {
                break;
            }
        }
        Ok(())
    }
}

impl SpecReader for GronReader {
    fn begin_object(&mut self) -> Result<(), SCodecError> { self.begin_object() }
    fn has_next_field(&mut self) -> Result<bool, SCodecError> { self.has_next_field() }
    fn read_field_name(&mut self) -> Result<String, SCodecError> { self.read_field_name() }
    fn end_object(&mut self) -> Result<(), SCodecError> { self.end_object() }
    fn begin_array(&mut self) -> Result<(), SCodecError> { self.begin_array() }
    fn has_next_element(&mut self) -> Result<bool, SCodecError> { self.has_next_element() }
    fn end_array(&mut self) -> Result<(), SCodecError> { self.end_array() }
    fn read_string(&mut self) -> Result<String, SCodecError> { self.read_string() }
    fn read_bool(&mut self) -> Result<bool, SCodecError> { self.read_bool() }
    fn read_int32(&mut self) -> Result<i32, SCodecError> { self.read_int32() }
    fn read_int64(&mut self) -> Result<i64, SCodecError> { self.read_int64() }
    fn read_uint32(&mut self) -> Result<u32, SCodecError> { self.read_uint32() }
    fn read_uint64(&mut self) -> Result<u64, SCodecError> { self.read_uint64() }
    fn read_float32(&mut self) -> Result<f32, SCodecError> { self.read_float32() }
    fn read_float64(&mut self) -> Result<f64, SCodecError> { self.read_float64() }
    fn read_null(&mut self) -> Result<(), SCodecError> { self.read_null() }
    fn read_bytes(&mut self) -> Result<Vec<u8>, SCodecError> { self.read_bytes() }
    fn read_enum(&mut self) -> Result<String, SCodecError> {
        let v = Self::unescape(&self.lines[self.cursor].1)?; self.cursor += 1; Ok(v)
    }
    fn is_null(&mut self) -> Result<bool, SCodecError> { self.is_null() }
    fn skip(&mut self) -> Result<(), SCodecError> { self.skip() }
}
