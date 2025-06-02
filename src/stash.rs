use crate::byte_reader::ByteReader;
use crate::stash_entry::StashEntry;

use std::fs::File;
use std::io::Error;
use std::path::PathBuf;
use std::io::Read;

const PRIME: u32 = 39916801;

struct Block {
    #[allow(dead_code)]
    len: u32,
    end: u32,
}

pub struct Decrypt {
    slice_reader: ByteReader,
    table: [u32; 256],
    key: u32,
}

impl Decrypt {
    pub fn new(path: &PathBuf) -> Result<Self, Error> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        let _len = file.read_to_end(&mut bytes)?;
        let mut byte_vec = ByteReader::from_slice(&bytes);
        let key = byte_vec.read_u32() ^ 0x55555555;
        let mut k = key;
        let mut table = [0; 256];
        for i in &mut table {
            k = k.rotate_right(1).wrapping_mul(PRIME);
            *i = k;
        }

        Ok(Self {
            slice_reader: byte_vec,
            table,
            key,
        })
    }

    pub fn read_int(&mut self) -> u32 {
        let num = self.slice_reader.read_u32();
        let ret = num ^ self.key;
        for byte in num.to_be_bytes() {
            self.key ^= self.table[byte as usize];
        }
        ret
    }

    fn next_int(&mut self) -> u32 {
        self.slice_reader.read_u32() ^ self.key
    }

    #[allow(dead_code)]
    fn next_float(&mut self) -> f32 {
        self.next_int() as f32
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.slice_reader.read_byte();
        self.key ^= self.table[byte as usize];
        byte ^ (self.key as u8)
    }

    fn read_bool(&mut self) -> bool {
        self.read_byte() != 0
    }

    pub fn read_str(&mut self) -> Result<String, Error> {
        let len = self.read_int();
        if len > 0 {
            let str_buf = self.slice_reader.read_n_bytes(len);
            for i in 0..len {
                let byte = (str_buf[i as usize] as u32 ^ self.key) as u8;
                self.key ^= self.table[str_buf[i as usize] as usize];
                str_buf[i as usize] = byte;
            }
            // TODO error handling for invalid strings
            let ret_str = str::from_utf8(str_buf).unwrap().to_string();
            return Ok(ret_str);
        }
        Ok("".to_string())
    }

    fn read_block_start(&mut self) -> (u32, Block) {
        let block_start = self.read_int();
        let len = self.next_int();
        let index: u32 = self.slice_reader.index.try_into().unwrap();
        let end = index + len;
        (block_start, Block { len, end })
    }

    fn read_block_end(&mut self, block: &Block) -> Result<bool, ()> {
        let stream_pos: u32 = self.slice_reader.index.try_into().unwrap();
        if block.end != stream_pos {
            println!(
                "Stream position is {stream_pos} but block end is {}. Delta: {}",
                block.end,
                stream_pos.abs_diff(block.end)
            );
            Err(())
        } else if self.next_int() != 0 {
            println!("Expected end of block character 0.");
            Ok(false)
        } else {
            Ok(true)
        }
    }
}

pub struct Stash {
    pub tabs: Vec<Vec<StashEntry>>,
}

impl Stash {
    #[allow(dead_code)]
    pub fn new(path: &PathBuf) -> Result<Self, Error> {
        let mut decrypt = Decrypt::new(path)?;
        let val = decrypt.read_int();
        assert_eq!(val, 2);
        let (block_pos, block) = decrypt.read_block_start();
        assert_eq!(block_pos, 18);
        let stash_version = decrypt.read_int();
        assert_eq!(stash_version, 5); // Stash file version 5
        assert_eq!(decrypt.next_int(), 0);
        let _str_mod = decrypt.read_str()?;
        //print!("{str_mod}");

        if stash_version >= 5 {
            let _has_expansion1 = decrypt.read_bool(); // does this refer to AoM?
            //println!("bool is {has_expansion1}");
        }

        let tabs_count = decrypt.read_int();
        let mut tabs = Vec::new();

        for _ in 0..tabs_count {
            let mut items = Vec::new();
            let (_block_start, block) = decrypt.read_block_start();
            let _stash_width = decrypt.read_int();
            let _stash_height = decrypt.read_int();
            let item_count = decrypt.read_int();

            for _ in 0..item_count {
                let item = StashEntry::read(&mut decrypt)?;
                items.push(item);
            }
            tabs.push(items);
            decrypt.read_block_end(&block).unwrap();
        }
        decrypt.read_block_end(&block).unwrap();

        Ok(Self { tabs })
    }
}
