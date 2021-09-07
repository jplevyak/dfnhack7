#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct u24(pub u32);

impl u24 {
    pub fn to_be_bytes(&self) -> [u8; 3] {
        let mut bytes: [u8; 3] = Default::default();
        bytes.copy_from_slice(&self.0.to_be_bytes()[1..4]);
        bytes
    }
}

impl From<usize> for u24 {
    #[inline]
    fn from(v: usize) -> Self {
        u24(v as u32)
    }
}

impl From<u24> for usize {
    #[inline]
    fn from(v: u24) -> Self {
        v.0 as Self
    }
}

pub struct Writer(pub Vec<u8>);

impl Writer {
    pub fn new() -> Self {
        Writer(vec![])
    }

    #[inline]
    pub fn write_u8(&mut self, v: u8) {
        self.0.push(v);
    }

    #[inline]
    pub fn write_u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_be_bytes());
    }

    #[inline]
    pub fn write_u24(&mut self, v: u24) {
        self.0.extend_from_slice(&v.to_be_bytes());
    }

    #[inline]
    pub fn write_u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_be_bytes());
    }

    #[inline]
    pub fn write_u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_be_bytes());
    }

    #[inline]
    pub fn append(&mut self, w: &[u8]) {
        self.0.extend(w);
    }

    #[inline]
    pub fn append_with_size_u8(&mut self, w: &[u8]) {
        self.write_u8(w.len() as u8);
        self.append(w);
    }

    #[inline]
    pub fn append_with_size_u16(&mut self, w: &[u8]) {
        self.write_u16(w.len() as u16);
        self.append(w);
    }

    #[inline]
    pub fn append_with_size_u24(&mut self, w: &[u8]) {
        self.write_u24(w.len().into());
        self.append(w);
    }

    #[inline]
    pub fn append_string(&mut self, s: &str) {
        self.0.extend(s.as_bytes());
    }

    #[inline]
    pub fn append_string_size_u8(&mut self, s: &str) {
        self.write_u8(s.len() as u8);
        self.0.extend(s.as_bytes());
    }

    #[inline]
    pub fn append_string_size_u16(&mut self, s: &str) {
        self.write_u16(s.len() as u16);
        self.0.extend(s.as_bytes());
    }
}

#[derive(Debug)]
pub struct Reader<'a> {
    pub data: &'a [u8],
    pub index: usize,
}

#[derive(Debug, Clone)]
pub enum ReaderError {
    InsufficentData,
}

impl<'a> Reader<'a> {
    pub fn from(data: &'a [u8], index: usize) -> Self {
        Reader { data, index }
    }

    #[inline]
    pub fn read_n(&mut self, n: usize) -> Result<&'a [u8], ReaderError> {
        if self.has(n) {
            let result = &self.data[self.index..self.index + n];
            self.index += n;
            Ok(result)
        } else {
            Err(ReaderError::InsufficentData)
        }
    }

    #[inline]
    pub fn read_u8(&mut self) -> Result<u8, ReaderError> {
        if self.has(1) {
            let result = self.data[self.index];
            self.index += 1;
            Ok(result)
        } else {
            Err(ReaderError::InsufficentData)
        }
    }

    #[inline]
    pub fn read_u16(&mut self) -> Result<u16, ReaderError> {
        if self.has(2) {
            let result = ((self.data[self.index] as u16) << 8) | (self.data[self.index + 1] as u16);
            self.index += 2;
            Ok(result)
        } else {
            Err(ReaderError::InsufficentData)
        }
    }

    #[inline]
    pub fn read_u24(&mut self) -> Result<u24, ReaderError> {
        if self.has(3) {
            let result = ((self.data[self.index] as u32) << 16)
                | ((self.data[self.index + 1] as u32) << 8)
                | (self.data[self.index + 2] as u32);
            self.index += 3;
            Ok(u24(result))
        } else {
            Err(ReaderError::InsufficentData)
        }
    }

    #[inline]
    pub fn read_u16_bytes(&mut self) -> Result<&'a [u8], ReaderError> {
        if self.has(2) {
            let size =
                ((self.data[self.index] as usize) << 8) | (self.data[self.index + 1] as usize);

            // println!("read_u16_bytes: {:?}", size);
            if self.has(2 + size) {
                self.index += 2 + size;

                Ok(&self.data[(self.index - size)..self.index])
            } else {
                Err(ReaderError::InsufficentData)
            }
        } else {
            Err(ReaderError::InsufficentData)
        }
    }

    #[inline]
    pub fn read_u24_bytes(&mut self) -> Result<&'a [u8], ReaderError> {
        if self.has(3) {
            let size = ((self.data[self.index] as usize) << 16)
                | ((self.data[self.index + 1] as usize) << 8)
                | (self.data[self.index + 2] as usize);

            // println!("read_u24_bytes: {:?}", size);
            if self.has(3 + size) {
                self.index += 3 + size;

                Ok(&self.data[(self.index - size)..self.index])
            } else {
                Err(ReaderError::InsufficentData)
            }
        } else {
            Err(ReaderError::InsufficentData)
        }
    }

    #[inline]
    pub fn read_u8_bytes(&mut self) -> Result<&'a [u8], ReaderError> {
        if self.has(1) {
            let size = self.data[self.index] as usize;

            // println!("read_u8_bytes: {:?}", size);
            if self.has(1 + size) {
                self.index += 1 + size;

                Ok(&self.data[(self.index - size)..self.index])
            } else {
                Err(ReaderError::InsufficentData)
            }
        } else {
            Err(ReaderError::InsufficentData)
        }
    }

    #[inline]
    pub fn rest(&mut self) -> &'a [u8] {
        &self.data[self.index..]
    }

    #[inline]
    pub fn has(&self, bytes_left: usize) -> bool {
        self.data.len() >= self.index + bytes_left
    }

    #[inline]
    pub fn size_left(&self) -> usize {
        self.data.len() - self.index
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_writer() {
        let mut w = Writer::new();
        assert_eq!(w.0.len(), 0);

        w.write_u8(0x00);
        assert_eq!(w.0.len(), 1);
        assert_eq!(w.0, vec![0x00 as u8]);

        w.write_u16(0x1122);
        assert_eq!(w.0.len(), 3);
        assert_eq!(w.0, vec![0x00 as u8, 0x11, 0x22]);

        w.write_u24(0x998877.into());
        assert_eq!(w.0.len(), 6);
        assert_eq!(w.0, vec![0x00 as u8, 0x11, 0x22, 0x99, 0x88, 0x77]);

        let mut w2 = Writer::new();
        w2.append_string("000");
        assert_eq!(w2.0.len(), "000".len());

        w.append_with_size_u16(&w2.0);
        assert_eq!(w.0.len(), 6 + 2 + "000".len());
        assert_eq!(
            w.0,
            vec![0x00 as u8, 0x11, 0x22, 0x99, 0x88, 0x77, 0x00, 0x03, 0x30, 0x30, 0x30]
        );
    }

    #[test]
    fn test_reader() {
        let mut data = vec![
            0x11, 0x22, 0x33, 0x44, 0x45, 0x46, 0x00, 0x03, 0x55, 0x66, 0x77,
        ];

        let mut reader = Reader::from(data.as_slice(), 0);

        let v = reader.read_u8();
        assert!(v.is_ok());
        assert_eq!(v.unwrap(), 0x11);

        let v = reader.read_u16();
        assert!(v.is_ok());
        assert_eq!(v.unwrap(), 0x2233);

        let v = reader.read_u24();
        assert!(v.is_ok());
        assert_eq!(v.unwrap().0, 0x444546);

        let v = reader.read_u16_bytes();
        assert!(v.is_ok());
        assert_eq!(v.unwrap(), vec![0x55, 0x66, 0x77]);

        let last_index = reader.index;

        let v = reader.read_u8();
        assert!(v.is_err());

        drop(reader);

        data.push(0x88);

        let mut reader = Reader::from(data.as_slice(), last_index);
        let v = reader.read_u8();
        assert!(v.is_ok());
        assert_eq!(v.unwrap(), 0x88);
    }
}
