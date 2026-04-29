#[derive(Debug)]
pub struct SCodecError {
    pub message: String,
}

impl SCodecError {
    pub fn new(msg: impl Into<String>) -> Self { SCodecError { message: msg.into() } }
}

impl std::fmt::Display for SCodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.message) }
}

impl std::error::Error for SCodecError {}

pub struct JsonReader<'a> {
    src: &'a str,
    pos: usize,
    first_field: Vec<bool>,
    first_elem: Vec<bool>,
}

impl<'a> JsonReader<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, SCodecError> {
        let s = std::str::from_utf8(data).map_err(|_| SCodecError::new("json: invalid utf-8"))?;
        Ok(JsonReader { src: s, pos: 0, first_field: Vec::new(), first_elem: Vec::new() })
    }

    pub fn pos(&self) -> usize { self.pos }

    fn ws(&mut self) {
        while self.pos < self.src.len() {
            match self.src.as_bytes()[self.pos] {
                0x20 | 0x09 | 0x0A | 0x0D => self.pos += 1,
                _ => break,
            }
        }
    }

    fn peek(&mut self) -> Result<u8, SCodecError> {
        self.ws();
        if self.pos >= self.src.len() { return Err(Self::eof()); }
        Ok(self.src.as_bytes()[self.pos])
    }

    fn read_ch(&mut self) -> Result<u8, SCodecError> {
        self.ws();
        if self.pos >= self.src.len() { return Err(Self::eof()); }
        let b = self.src.as_bytes()[self.pos];
        self.pos += 1;
        Ok(b)
    }

    fn expect(&mut self, ch: u8) -> Result<(), SCodecError> {
        let got = self.read_ch()?;
        if got != ch { return Err(SCodecError::new(format!("json: expected '{}', got '{}' at {}", ch as char, got as char, self.pos - 1))); }
        Ok(())
    }

    fn eof() -> SCodecError {
        SCodecError::new("json: unexpected end of input")
    }

    fn parse_string(&mut self) -> Result<String, SCodecError> {
        self.expect(b'"')?;
        let mut result = String::new();
        while self.pos < self.src.len() {
            let c = self.src.as_bytes()[self.pos];
            if c == b'"' { self.pos += 1; return Ok(result); }
            if c == b'\\' {
                self.pos += 1;
                if self.pos >= self.src.len() { return Err(Self::eof()); }
                let esc = self.src.as_bytes()[self.pos];
                match esc {
                    b'"' => { result.push('"'); self.pos += 1; }
                    b'\\' => { result.push('\\'); self.pos += 1; }
                    b'/' => { result.push('/'); self.pos += 1; }
                    b'b' => { result.push('\x08'); self.pos += 1; }
                    b'f' => { result.push('\x0C'); self.pos += 1; }
                    b'n' => { result.push('\n'); self.pos += 1; }
                    b'r' => { result.push('\r'); self.pos += 1; }
                    b't' => { result.push('\t'); self.pos += 1; }
                    b'u' => {
                        self.pos += 1;
                        if self.pos + 4 > self.src.len() { return Err(SCodecError::new("json: incomplete unicode escape")); }
                        let hex = &self.src[self.pos..self.pos + 4];
                        let cp = u32::from_str_radix(hex, 16).map_err(|_| SCodecError::new(format!("json: invalid unicode escape \\u{}", hex)))?;
                        self.pos += 4;
                        let mut cp = cp as u32;
                        if cp >= 0xD800 && cp <= 0xDBFF {
                            if self.pos + 6 <= self.src.len() && self.src.as_bytes()[self.pos] == b'\\' && self.src.as_bytes()[self.pos + 1] == b'u' {
                                self.pos += 2;
                                let hex2 = &self.src[self.pos..self.pos + 4];
                                let low = u32::from_str_radix(hex2, 16).map_err(|_| SCodecError::new(format!("json: invalid low surrogate \\u{}", hex2)))?;
                                self.pos += 4;
                                if low >= 0xDC00 && low <= 0xDFFF {
                                    cp = 0x10000 + (cp - 0xD800) * 0x400 + (low - 0xDC00);
                                } else {
                                    return Err(SCodecError::new("json: expected low surrogate"));
                                }
                            } else {
                                return Err(SCodecError::new("json: expected low surrogate"));
                            }
                        }
                        if let Some(ch) = char::from_u32(cp) { result.push(ch); }
                    }
                    _ => return Err(SCodecError::new(format!("json: invalid escape character '\\{}'", esc as char))),
                }
            } else if c < 0x20 {
                return Err(SCodecError::new(format!("json: unescaped control character U+{:04X}", c)));
            } else {
                result.push(c as char);
                self.pos += 1;
            }
        }
        Err(SCodecError::new("json: unterminated string"))
    }

    fn parse_number_raw(&mut self) -> Result<&'a str, SCodecError> {
        let start = self.pos;
        if self.pos < self.src.len() && self.src.as_bytes()[self.pos] == b'-' { self.pos += 1; }
        if self.pos >= self.src.len() { return Err(SCodecError::new("json: unexpected end of number")); }
        if self.src.as_bytes()[self.pos] == b'0' {
            self.pos += 1;
        } else if self.src.as_bytes()[self.pos] >= b'1' && self.src.as_bytes()[self.pos] <= b'9' {
            self.pos += 1;
            while self.pos < self.src.len() && self.src.as_bytes()[self.pos] >= b'0' && self.src.as_bytes()[self.pos] <= b'9' { self.pos += 1; }
        } else {
            return Err(SCodecError::new("json: invalid number"));
        }
        if self.pos < self.src.len() && self.src.as_bytes()[self.pos] == b'.' {
            self.pos += 1;
            if self.pos >= self.src.len() || self.src.as_bytes()[self.pos] < b'0' || self.src.as_bytes()[self.pos] > b'9' {
                return Err(SCodecError::new("json: invalid number fraction"));
            }
            while self.pos < self.src.len() && self.src.as_bytes()[self.pos] >= b'0' && self.src.as_bytes()[self.pos] <= b'9' { self.pos += 1; }
        }
        if self.pos < self.src.len() && (self.src.as_bytes()[self.pos] == b'e' || self.src.as_bytes()[self.pos] == b'E') {
            self.pos += 1;
            if self.pos < self.src.len() && (self.src.as_bytes()[self.pos] == b'+' || self.src.as_bytes()[self.pos] == b'-') { self.pos += 1; }
            if self.pos >= self.src.len() || self.src.as_bytes()[self.pos] < b'0' || self.src.as_bytes()[self.pos] > b'9' {
                return Err(SCodecError::new("json: invalid number exponent"));
            }
            while self.pos < self.src.len() && self.src.as_bytes()[self.pos] >= b'0' && self.src.as_bytes()[self.pos] <= b'9' { self.pos += 1; }
        }
        Ok(&self.src[start..self.pos])
    }

    pub fn read_string(&mut self) -> Result<String, SCodecError> { self.parse_string() }

    pub fn read_bool(&mut self) -> Result<bool, SCodecError> {
        let ch = self.peek()?;
        match ch {
            b't' => { for &c in b"true" { if self.read_ch()? != c { return Err(SCodecError::new("json: expected true")); } } Ok(true) }
            b'f' => { for &c in b"false" { if self.read_ch()? != c { return Err(SCodecError::new("json: expected false")); } } Ok(false) }
            _ => Err(SCodecError::new(format!("json: expected bool, got '{}'", ch as char))),
        }
    }

    pub fn read_int32(&mut self) -> Result<i32, SCodecError> {
        let raw = self.parse_number_raw()?;
        raw.parse::<i32>().map_err(|_| SCodecError::new(format!("json: invalid int32: {}", raw)))
    }

    pub fn read_int64(&mut self) -> Result<i64, SCodecError> {
        let ch = self.peek()?;
        if ch == b'"' {
            let s = self.parse_string()?;
            s.parse::<i64>().map_err(|_| SCodecError::new(format!("json: invalid int64: {}", s)))
        } else {
            let raw = self.parse_number_raw()?;
            raw.parse::<i64>().map_err(|_| SCodecError::new(format!("json: invalid int64: {}", raw)))
        }
    }

    pub fn read_uint32(&mut self) -> Result<u32, SCodecError> {
        let raw = self.parse_number_raw()?;
        raw.parse::<u32>().map_err(|_| SCodecError::new(format!("json: invalid uint32: {}", raw)))
    }

    pub fn read_uint64(&mut self) -> Result<u64, SCodecError> {
        let ch = self.peek()?;
        if ch == b'"' {
            let s = self.parse_string()?;
            s.parse::<u64>().map_err(|_| SCodecError::new(format!("json: invalid uint64: {}", s)))
        } else {
            let raw = self.parse_number_raw()?;
            raw.parse::<u64>().map_err(|_| SCodecError::new(format!("json: invalid uint64: {}", raw)))
        }
    }

    pub fn read_float32(&mut self) -> Result<f32, SCodecError> {
        let raw = self.parse_number_raw()?;
        raw.parse::<f32>().map_err(|_| SCodecError::new(format!("json: invalid float32: {}", raw)))
    }

    pub fn read_float64(&mut self) -> Result<f64, SCodecError> {
        let raw = self.parse_number_raw()?;
        raw.parse::<f64>().map_err(|_| SCodecError::new(format!("json: invalid float64: {}", raw)))
    }

    pub fn read_null(&mut self) -> Result<(), SCodecError> {
        for &c in b"null" { if self.read_ch()? != c { return Err(SCodecError::new("json: expected null")); } }
        Ok(())
    }

    pub fn read_bytes(&mut self) -> Result<Vec<u8>, SCodecError> {
        let s = self.parse_string()?;
        Self::b64_decode(&s)
    }

    fn b64_decode(s: &str) -> Result<Vec<u8>, SCodecError> {
        if s.len() % 4 != 0 { return Err(SCodecError::new("json: invalid base64 length")); }
        const LOOKUP: &[i8] = &[
            -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,
            -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,
            -1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,62,-1,-1,-1,63,
            52,53,54,55,56,57,58,59,60,61,-1,-1,-1,-1,-1,-1,
            -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,10,11,12,13,14,
            15,16,17,18,19,20,21,22,23,24,25,-1,-1,-1,-1,-1,
            -1,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,
            41,42,43,44,45,46,47,48,49,50,51,-1,-1,-1,-1,-1,
        ];
        let mut buf = Vec::with_capacity(s.len() * 3 / 4);
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let a = LOOKUP[bytes[i] as usize];
            let b = LOOKUP[bytes[i + 1] as usize];
            let c = if bytes[i + 2] == b'=' { -1i8 } else { LOOKUP[bytes[i + 2] as usize] };
            let d = if bytes[i + 3] == b'=' { -1i8 } else { LOOKUP[bytes[i + 3] as usize] };
            if a < 0 || b < 0 { return Err(SCodecError::new("json: invalid base64")); }
            buf.push(((a as u8) << 2) | ((b as u8) >> 4));
            if c >= 0 {
                buf.push(((b as u8 & 0xF) << 4) | ((c as u8) >> 2));
                if d >= 0 {
                    buf.push(((c as u8 & 0x3) << 6) | (d as u8));
                }
            }
            i += 4;
        }
        Ok(buf)
    }

    pub fn read_enum(&mut self) -> Result<String, SCodecError> { self.parse_string() }

    pub fn begin_object(&mut self) -> Result<(), SCodecError> { self.expect(b'{') }

    pub fn has_next_field(&mut self) -> Result<bool, SCodecError> {
        Ok(self.peek()? != b'}')
    }

    pub fn read_field_name(&mut self) -> Result<String, SCodecError> { self.parse_string() }

    pub fn next_field_separator(&mut self) -> Result<(), SCodecError> {
        let ch = self.peek()?;
        if ch == b',' { self.pos += 1; }
        else if ch != b'}' { return Err(SCodecError::new(format!("json: expected ',' or '}}', got '{}'", ch as char))); }
        Ok(())
    }

    pub fn end_object(&mut self) -> Result<(), SCodecError> { self.expect(b'}') }

    pub fn begin_array(&mut self) -> Result<(), SCodecError> { self.expect(b'[') }

    pub fn has_next_element(&mut self) -> Result<bool, SCodecError> {
        Ok(self.peek()? != b']')
    }

    pub fn next_element_separator(&mut self) -> Result<(), SCodecError> {
        let ch = self.peek()?;
        if ch == b',' { self.pos += 1; }
        else if ch != b']' { return Err(SCodecError::new(format!("json: expected ',' or ']', got '{}'", ch as char))); }
        Ok(())
    }

    pub fn end_array(&mut self) -> Result<(), SCodecError> { self.expect(b']') }

    pub fn is_null(&mut self) -> Result<bool, SCodecError> {
        Ok(self.peek()? == b'n')
    }

    pub fn skip(&mut self) -> Result<(), SCodecError> {
        self.ws();
        if self.pos >= self.src.len() { return Err(Self::eof()); }
        let ch = self.src.as_bytes()[self.pos];
        match ch {
            b'"' => {
                self.pos += 1;
                while self.pos < self.src.len() {
                    if self.src.as_bytes()[self.pos] == b'\\' { self.pos += 2; }
                    else if self.src.as_bytes()[self.pos] == b'"' { self.pos += 1; return Ok(()); }
                    else { self.pos += 1; }
                }
                Err(SCodecError::new("json: unterminated string in skip"))
            }
            b'{' => {
                use crate::spec_reader::SpecReader;
                SpecReader::begin_object(self)?;
                while SpecReader::has_next_field(self)? {
                    SpecReader::read_field_name(self)?;
                    self.skip()?;
                }
                SpecReader::end_object(self)
            }
            b'[' => {
                use crate::spec_reader::SpecReader;
                SpecReader::begin_array(self)?;
                while SpecReader::has_next_element(self)? {
                    self.skip()?;
                }
                SpecReader::end_array(self)
            }
            b't' => { for &c in b"true" { if self.read_ch()? != c { return Err(SCodecError::new("json: skip expected true")); } } Ok(()) }
            b'f' => { for &c in b"false" { if self.read_ch()? != c { return Err(SCodecError::new("json: skip expected false")); } } Ok(()) }
            b'n' => { for &c in b"null" { if self.read_ch()? != c { return Err(SCodecError::new("json: skip expected null")); } } Ok(()) }
            _ => {
                if (ch >= b'0' && ch <= b'9') || ch == b'-' {
                    self.parse_number_raw()?;
                     Ok(())
                } else {
                    Err(SCodecError::new(format!("json: unexpected '{}' in skip", ch as char)))
                }
            }
        }
    }
}

impl crate::spec_reader::SpecReader for JsonReader<'_> {
    fn begin_object(&mut self) -> Result<(), SCodecError> {
        self.expect(b'{')?;
        self.first_field.push(true);
        Ok(())
    }

    fn has_next_field(&mut self) -> Result<bool, SCodecError> {
        let ch = self.peek()?;
        if ch == b'}' {
            self.first_field.pop();
            return Ok(false);
        }
        let top = self.first_field.len() - 1;
        if !self.first_field[top] {
            if ch != b',' { return Err(SCodecError::new(format!("json: expected ',' or '}}', got '{}'", ch as char))); }
            self.pos += 1;
        } else {
            self.first_field[top] = false;
        }
        Ok(true)
    }

    fn read_field_name(&mut self) -> Result<String, SCodecError> {
        let key = self.parse_string()?;
        self.ws();
        if self.pos < self.src.len() && self.src.as_bytes()[self.pos] == b':' {
            self.pos += 1;
        } else {
            return Err(SCodecError::new(format!("json: expected ':' after field name '{}'", key)));
        }
        Ok(key)
    }

    fn end_object(&mut self) -> Result<(), SCodecError> { self.expect(b'}') }

    fn begin_array(&mut self) -> Result<(), SCodecError> {
        self.expect(b'[')?;
        self.first_elem.push(true);
        Ok(())
    }

    fn has_next_element(&mut self) -> Result<bool, SCodecError> {
        let ch = self.peek()?;
        if ch == b']' {
            self.first_elem.pop();
            return Ok(false);
        }
        let top = self.first_elem.len() - 1;
        if !self.first_elem[top] {
            if ch != b',' { return Err(SCodecError::new(format!("json: expected ',' or ']', got '{}'", ch as char))); }
            self.pos += 1;
        } else {
            self.first_elem[top] = false;
        }
        Ok(true)
    }

    fn end_array(&mut self) -> Result<(), SCodecError> { self.expect(b']') }

    fn read_string(&mut self) -> Result<String, SCodecError> { self.parse_string() }
    fn read_bool(&mut self) -> Result<bool, SCodecError> { self.read_bool() }
    fn read_int32(&mut self) -> Result<i32, SCodecError> { self.read_int32() }
    fn read_int64(&mut self) -> Result<i64, SCodecError> { self.read_int64() }
    fn read_uint32(&mut self) -> Result<u32, SCodecError> { self.read_uint32() }
    fn read_uint64(&mut self) -> Result<u64, SCodecError> { self.read_uint64() }
    fn read_float32(&mut self) -> Result<f32, SCodecError> { self.read_float32() }
    fn read_float64(&mut self) -> Result<f64, SCodecError> { self.read_float64() }
    fn read_null(&mut self) -> Result<(), SCodecError> { self.read_null() }
    fn read_bytes(&mut self) -> Result<Vec<u8>, SCodecError> { self.read_bytes() }
    fn read_enum(&mut self) -> Result<String, SCodecError> { self.parse_string() }
    fn is_null(&mut self) -> Result<bool, SCodecError> { self.is_null() }
    fn skip(&mut self) -> Result<(), SCodecError> { self.skip() }
}
