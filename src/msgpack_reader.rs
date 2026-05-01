use crate::json_reader::SCodecError;

pub struct MsgPackReader {
    data: Vec<u8>,
    pos: usize,
    container_count: Vec<usize>,
}

impl MsgPackReader {
    pub fn new(data: &[u8]) -> Self {
        MsgPackReader { data: data.to_vec(), pos: 0, container_count: Vec::new() }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    fn read_byte(&mut self) -> Result<u8, SCodecError> {
        if self.pos >= self.data.len() {
            return Err(SCodecError::new("msgpack: unexpected end of data"));
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    fn read_u16(&mut self) -> Result<u16, SCodecError> {
        if self.pos + 2 > self.data.len() { return self.eof(); }
        let v = u16::from_be_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Ok(v)
    }

    fn read_u32(&mut self) -> Result<u32, SCodecError> {
        if self.pos + 4 > self.data.len() { return self.eof(); }
        let v = u32::from_be_bytes([
            self.data[self.pos], self.data[self.pos + 1],
            self.data[self.pos + 2], self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(v)
    }

    fn read_i16(&mut self) -> Result<i16, SCodecError> { Ok(self.read_u16()? as i16) }
    fn read_i32(&mut self) -> Result<i32, SCodecError> { Ok(self.read_u32()? as i32) }
    fn read_u64(&mut self) -> Result<u64, SCodecError> {
        if self.pos + 8 > self.data.len() { return self.eof(); }
        let v = u64::from_be_bytes([
            self.data[self.pos], self.data[self.pos+1], self.data[self.pos+2], self.data[self.pos+3],
            self.data[self.pos+4], self.data[self.pos+5], self.data[self.pos+6], self.data[self.pos+7],
        ]);
        self.pos += 8;
        Ok(v)
    }
    fn read_i64(&mut self) -> Result<i64, SCodecError> { Ok(self.read_u64()? as i64) }

    fn eof<T>(&self) -> Result<T, SCodecError> {
        Err(SCodecError::new("msgpack: unexpected end of data"))
    }

    pub fn read_map_header(&mut self) -> Result<usize, SCodecError> {
        let b = self.read_byte()?;
        if b & 0xF0 == 0x80 { return Ok((b & 0x0F) as usize); }
        if b == 0xDE { return Ok(self.read_u16()? as usize); }
        if b == 0xDF { return Ok(self.read_u32()? as usize); }
        Err(SCodecError::new(format!("msgpack: expected map, got 0x{b:02X}")))
    }

    pub fn read_array_header(&mut self) -> Result<usize, SCodecError> {
        let b = self.read_byte()?;
        if b & 0xF0 == 0x90 { return Ok((b & 0x0F) as usize); }
        if b == 0xDC { return Ok(self.read_u16()? as usize); }
        if b == 0xDD { return Ok(self.read_u32()? as usize); }
        Err(SCodecError::new(format!("msgpack: expected array, got 0x{b:02X}")))
    }

    pub fn read_string(&mut self) -> Result<String, SCodecError> {
        let b = self.read_byte()?;
        let len: usize = match b {
            b if b & 0xE0 == 0xA0 => (b & 0x1F) as usize,
            0xD9 => self.read_byte()? as usize,
            0xDA => self.read_u16()? as usize,
            0xDB => self.read_u32()? as usize,
            _ => return Err(SCodecError::new(format!("msgpack: expected string, got 0x{b:02X}"))),
        };
        if self.pos + len > self.data.len() { return self.eof(); }
        let s = String::from_utf8_lossy(&self.data[self.pos..self.pos + len]).into_owned();
        self.pos += len;
        Ok(s)
    }

    pub fn read_int(&mut self) -> Result<i64, SCodecError> {
        let b = self.read_byte()?;
        match b {
            b if b <= 0x7F => Ok(b as i64),
            b if b >= 0xE0 => Ok((b as i8) as i64),
            0xCC => Ok(self.read_byte()? as i64),
            0xCD => Ok(self.read_u16()? as i64),
            0xCE => Ok(self.read_u32()? as i64),
            0xD0 => Ok(self.read_byte()? as i8 as i64),
            0xD1 => Ok(self.read_i16()? as i64),
            0xD2 => Ok(self.read_i32()? as i64),
            0xD3 => Ok(self.read_i64()?),
            0xCF => Ok(self.read_u64()? as i64),
            _ => Err(SCodecError::new(format!("msgpack: expected int, got 0x{b:02X}"))),
        }
    }

    pub fn read_float(&mut self) -> Result<f64, SCodecError> {
        let b = self.read_byte()?;
        match b {
            0xCA => {
                if self.pos + 4 > self.data.len() { return self.eof(); }
                let bits = u32::from_be_bytes([
                    self.data[self.pos], self.data[self.pos + 1],
                    self.data[self.pos + 2], self.data[self.pos + 3],
                ]);
                self.pos += 4;
                Ok(f32::from_bits(bits) as f64)
            }
            0xCB => {
                if self.pos + 8 > self.data.len() { return self.eof(); }
                let bits = u64::from_be_bytes([
                    self.data[self.pos], self.data[self.pos + 1],
                    self.data[self.pos + 2], self.data[self.pos + 3],
                    self.data[self.pos + 4], self.data[self.pos + 5],
                    self.data[self.pos + 6], self.data[self.pos + 7],
                ]);
                self.pos += 8;
                Ok(f64::from_bits(bits))
            }
            _ => {
                self.pos -= 1;
                Ok(self.read_int()? as f64)
            }
        }
    }

    pub fn read_bool(&mut self) -> Result<bool, SCodecError> {
        let b = self.read_byte()?;
        match b {
            0xC3 => Ok(true),
            0xC2 => Ok(false),
            _ => Err(SCodecError::new(format!("msgpack: expected bool, got 0x{b:02X}"))),
        }
    }

    pub fn read_null(&mut self) -> Result<(), SCodecError> {
        let b = self.read_byte()?;
        if b == 0xC0 { Ok(()) } else { Err(SCodecError::new(format!("msgpack: expected null, got 0x{b:02X}"))) }
    }

    pub fn is_null(&self) -> bool {
        self.pos < self.data.len() && self.data[self.pos] == 0xC0
    }

    pub fn skip(&mut self) -> Result<(), SCodecError> {
        let b = self.read_byte()?;
        if b <= 0x7F || b >= 0xE0 { return Ok(()); }
        if b & 0xF0 == 0x80 {
            let n = (b & 0x0F) as usize;
            for _ in 0..n { self.skip()?; self.skip()?; }
            return Ok(());
        }
        if b & 0xF0 == 0x90 {
            let n = (b & 0x0F) as usize;
            for _ in 0..n { self.skip()?; }
            return Ok(());
        }
        if b & 0xE0 == 0xA0 {
            self.pos += (b & 0x1F) as usize;
            return Ok(());
        }
        match b {
            0xC0 | 0xC2 | 0xC3 => {}
            0xCC | 0xD0 => { self.pos += 1; }
            0xCD | 0xD1 => { self.pos += 2; }
            0xCE | 0xD2 | 0xCA => { self.pos += 4; }
            0xCF | 0xD3 | 0xCB => { self.pos += 8; }
            0xD9 => { self.pos += self.read_byte()? as usize; }
            0xDA => { self.pos += self.read_u16()? as usize; }
            0xDB => { self.pos += self.read_u32()? as usize; }
            0xC4 => { self.pos += self.read_byte()? as usize; }
            0xC5 => { self.pos += self.read_u16()? as usize; }
            0xC6 => { self.pos += self.read_u32()? as usize; }
            0xD4 => { self.pos += 2; }
            0xD5 => { self.pos += 3; }
            0xD6 => { self.pos += 5; }
            0xD7 => { self.pos += 9; }
            0xD8 => { self.pos += 17; }
            0xC7 => { self.pos += 1 + self.read_byte()? as usize; }
            0xC8 => { self.pos += 1 + self.read_u16()? as usize; }
            0xC9 => { self.pos += 1 + self.read_u32()? as usize; }
            0xDC => { let n = self.read_u16()? as usize; for _ in 0..n { self.skip()?; } }
            0xDD => { let n = self.read_u32()? as usize; for _ in 0..n { self.skip()?; } }
            0xDE => { let n = self.read_u16()? as usize; for _ in 0..n { self.skip()?; self.skip()?; } }
            0xDF => { let n = self.read_u32()? as usize; for _ in 0..n { self.skip()?; self.skip()?; } }
            _ => return Err(SCodecError::new(format!("msgpack: unknown format 0x{b:02X}"))),
        }
        Ok(())
    }

    pub fn begin_object(&mut self) -> Result<(), SCodecError> {
        let n = self.read_map_header()?;
        self.container_count.push(n);
        Ok(())
    }

    pub fn has_next_field(&mut self) -> Result<bool, SCodecError> {
        let top = self.container_count.len() - 1;
        if self.container_count[top] > 0 {
            self.container_count[top] -= 1;
            Ok(true)
        } else {
            self.container_count.pop();
            Ok(false)
        }
    }

    pub fn read_field_name(&mut self) -> Result<String, SCodecError> { self.read_string() }
    pub fn end_object(&mut self) -> Result<(), SCodecError> { Ok(()) }

    pub fn begin_array(&mut self) -> Result<(), SCodecError> {
        let n = self.read_array_header()?;
        self.container_count.push(n);
        Ok(())
    }

    pub fn has_next_element(&mut self) -> Result<bool, SCodecError> {
        let top = self.container_count.len() - 1;
        if self.container_count[top] > 0 {
            self.container_count[top] -= 1;
            Ok(true)
        } else {
            self.container_count.pop();
            Ok(false)
        }
    }

    pub fn end_array(&mut self) -> Result<(), SCodecError> { Ok(()) }

    pub fn read_bytes_raw(&mut self) -> Result<Vec<u8>, SCodecError> {
        let b = self.read_byte()?;
        let len = match b {
            0xC4 => self.read_byte()? as usize,
            0xC5 => self.read_u16()? as usize,
            0xC6 => self.read_u32()? as usize,
            _ => return Err(SCodecError::new(format!("msgpack: expected bin, got 0x{b:02X}"))),
        };
        if self.pos + len > self.data.len() { return self.eof(); }
        let v = self.data[self.pos..self.pos + len].to_vec();
        self.pos += len;
        Ok(v)
    }
}

impl crate::spec_reader::SpecReader for MsgPackReader {
    fn begin_object(&mut self) -> Result<(), SCodecError> {
        let n = self.read_map_header()?;
        self.container_count.push(n);
        Ok(())
    }

    fn has_next_field(&mut self) -> Result<bool, SCodecError> {
        let top = self.container_count.len() - 1;
        if self.container_count[top] > 0 {
            self.container_count[top] -= 1;
            Ok(true)
        } else {
            self.container_count.pop();
            Ok(false)
        }
    }

    fn read_field_name(&mut self) -> Result<String, SCodecError> { self.read_string() }
    fn end_object(&mut self) -> Result<(), SCodecError> { Ok(()) }

    fn begin_array(&mut self) -> Result<(), SCodecError> {
        let n = self.read_array_header()?;
        self.container_count.push(n);
        Ok(())
    }

    fn has_next_element(&mut self) -> Result<bool, SCodecError> {
        let top = self.container_count.len() - 1;
        if self.container_count[top] > 0 {
            self.container_count[top] -= 1;
            Ok(true)
        } else {
            self.container_count.pop();
            Ok(false)
        }
    }

    fn end_array(&mut self) -> Result<(), SCodecError> { Ok(()) }

    fn read_string(&mut self) -> Result<String, SCodecError> { self.read_string() }
    fn read_bool(&mut self) -> Result<bool, SCodecError> { self.read_bool() }
    fn read_int32(&mut self) -> Result<i32, SCodecError> { Ok(self.read_int()? as i32) }
    fn read_int64(&mut self) -> Result<i64, SCodecError> { self.read_int() }
    fn read_uint32(&mut self) -> Result<u32, SCodecError> { Ok(self.read_int()? as u32) }
    fn read_uint64(&mut self) -> Result<u64, SCodecError> { Ok(self.read_int()? as u64) }
    fn read_float32(&mut self) -> Result<f32, SCodecError> { Ok(self.read_float()? as f32) }
    fn read_float64(&mut self) -> Result<f64, SCodecError> { self.read_float() }
    fn read_null(&mut self) -> Result<(), SCodecError> { self.read_null() }
    fn read_bytes(&mut self) -> Result<Vec<u8>, SCodecError> { self.read_bytes_raw() }
    fn read_enum(&mut self) -> Result<String, SCodecError> { self.read_string() }
    fn is_null(&mut self) -> Result<bool, SCodecError> { Ok(MsgPackReader::is_null(self)) }
    fn skip(&mut self) -> Result<(), SCodecError> { self.skip() }
}
