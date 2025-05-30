mod item;
use item::GDItem;

use std::fs::File;
use std::io::Error;
use std::io::Read;
use std::path::PathBuf;

const PRIME: u32 = 39916801;

struct Block {
    #[allow(dead_code)]
    len: u32,
    end: u32,
}

struct ByteVec {
    bytes: Vec<u8>,
    index: usize,
}

impl ByteVec {
    fn new(path: &PathBuf) -> Result<Self, Error> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        let _len = file.read_to_end(&mut bytes)?;
        Ok(Self { bytes, index: 0 })
    }

    fn read_byte(&mut self) -> u8 {
        let ret = self.bytes[self.index];
        self.index += 1;
        ret
    }

    fn read_int(&mut self) -> u32 {
        let new_index = self.index + 4;
        let ret =
            u32::from_ne_bytes(<[u8; 4]>::try_from(&self.bytes[self.index..new_index]).unwrap());
        self.index = new_index;
        ret
    }

    fn read_n_bytes(&mut self, n: u32) -> &mut [u8] {
        let n = n as usize;
        let ret = &mut self.bytes[self.index..self.index + n];
        self.index += n;
        ret
    }
}

struct Decrypt {
    byte_vec: ByteVec,
    table: [u32; 256],
    key: u32,
}

impl Decrypt {
    pub fn new(path: &PathBuf) -> Result<Self, Error> {
        let mut byte_vec = ByteVec::new(path)?;
        let key = byte_vec.read_int() ^ 0x55555555;
        let mut k = key;
        let mut table = [0; 256];
        for i in &mut table {
            k = k.rotate_right(1).wrapping_mul(PRIME);
            *i = k;
        }

        Ok(Self {
            byte_vec,
            table,
            key,
        })
    }

    fn read_int(&mut self) -> u32 {
        let num = self.byte_vec.read_int();
        let ret = num ^ self.key;
        for byte in num.to_be_bytes() {
            self.key ^= self.table[byte as usize];
        }
        ret
    }

    fn next_int(&mut self) -> u32 {
        self.byte_vec.read_int() ^ self.key
    }

    #[allow(dead_code)]
    fn next_float(&mut self) -> f32 {
        self.next_int() as f32
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.byte_vec.read_byte();
        self.key ^= self.table[byte as usize];
        byte ^ (self.key as u8)
    }

    fn read_bool(&mut self) -> bool {
        self.read_byte() != 0
    }

    fn read_str(&mut self) -> Result<String, Error> {
        let len = self.read_int();
        if len > 0 {
            let str_buf = self.byte_vec.read_n_bytes(len);
            for i in 0..len {
                let byte = (str_buf[i as usize] as u32 ^ self.key) as u8;
                self.key ^= self.table[str_buf[i as usize] as usize];
                str_buf[i as usize] = byte;
            }
            // TODO error handling
            let ret_str = str::from_utf8(str_buf).unwrap().to_string();
            return Ok(ret_str);
        }
        Ok("".to_string())
    }

    fn read_block_start(&mut self) -> (u32, Block) {
        let block_start = self.read_int();
        let len = self.next_int();
        let index: u32 = self.byte_vec.index.try_into().unwrap();
        let end = index + len;
        (block_start, Block { len, end })
    }

    fn read_block_end(&mut self, block: &Block) -> Result<bool, ()> {
        let stream_pos: u32 = self.byte_vec.index.try_into().unwrap();
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

struct Stash {
    tabs: Vec<Vec<GDItem>>,
}

impl Stash {
    fn new(path: &PathBuf) -> Result<Self, Error> {
        let mut decrypt = Decrypt::new(path)?;
        let val = decrypt.read_int();
        assert_eq!(val, 2);
        println!("key {}", decrypt.key);

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
                let item = GDItem::read(&mut decrypt)?;
                items.push(item);
            }
            tabs.push(items);
            decrypt.read_block_end(&block).unwrap();
        }
        decrypt.read_block_end(&block).unwrap();

        Ok(Self { tabs })
    }
}

fn main() -> Result<(), Error> {
    let stash_path = std::path::PathBuf::from("transfer.gst");

    let stash = Stash::new(&stash_path)?;

    for tab in stash.tabs {
        for item in tab {
            println!("{:?}", item)
        }
    }

    Ok(())
}
