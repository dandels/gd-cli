use std::io::Error;
use std::io::Read;
use std::fs::File;
use std::path::PathBuf;

pub struct ByteReader {
    pub bytes: Vec<u8>,
    pub index: usize,
}

impl ByteReader {
    pub fn from_file(path: &PathBuf) -> Result<Self, Error> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        let _len = file.read_to_end(&mut bytes)?;
        Ok(Self { bytes, index: 0 })
    }

    /* Creating a vec for this seems wasteful, but I don't want to figure out how to deal with the lifetime issues of a
     * &[u8] */
    pub fn from_slice(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.to_vec(),
            index: 0
        }
    }

    pub fn read_byte(&mut self) -> u8 {
        let ret = self.bytes[self.index];
        self.index += 1;
        ret
    }

    pub fn read_u16(&mut self) -> u16 {
        let new_index = self.index + 2;
        let ret =
            u16::from_ne_bytes(<[u8; 2]>::try_from(&self.bytes[self.index..new_index]).unwrap());
        self.index = new_index;
        ret
    }

    pub fn read_u32(&mut self) -> u32 {
        let new_index = self.index + 4;
        let ret =
            u32::from_ne_bytes(<[u8; 4]>::try_from(&self.bytes[self.index..new_index]).unwrap());
        self.index = new_index;
        ret
    }

    pub fn read_f32(&mut self) -> f32 {
        let new_index = self.index + 4;
        let ret =
            f32::from_ne_bytes(<[u8; 4]>::try_from(&self.bytes[self.index..new_index]).unwrap());
        self.index = new_index;
        ret
    }



    pub fn read_u64(&mut self) -> u64 {
        let new_index = self.index + 8;
        let ret =
            u64::from_ne_bytes(<[u8; 8]>::try_from(&self.bytes[self.index..new_index]).unwrap());
        self.index = new_index;
        ret
    }

    pub fn read_n_bytes(&mut self, n: u32) -> &mut [u8] {
        let n = n as usize;
        let ret = &mut self.bytes[self.index..self.index + n];
        self.index += n;
        ret
    }

    pub fn read_string(&mut self, len: u32) -> String {
        str::from_utf8(self.read_n_bytes(len)).unwrap().to_string()
    }

    pub fn read_null_string(&mut self) -> Option<String> {
        if self.index >= self.bytes.len() {
            return None
        }
        let mut buf = Vec::new();
        while self.index < self.bytes.len() {
            let byte = self.read_byte();
            if byte == 0 {
                break;
            }
            buf.push(byte);
        }
        Some(String::from_utf8_lossy(&buf).to_string())
    }
}
