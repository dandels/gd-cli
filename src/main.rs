mod item;
use item::GDItem;

use std::fs::File;
use std::io::Error;
use std::io::Read;
use std::io::Seek;
use std::path::PathBuf;

const PRIME: u32 = 39916801;

struct Block {
    #[allow(dead_code)]
    len: u32,
    end: u32,
}

struct Decrypt {
    file: File,
    table: [u32; 256],
    key: u32,
    buf: [u8; 4],
}

impl Decrypt {
    pub fn new(name: &PathBuf) -> Result<Self, Error> {
        let mut file = File::open(name).unwrap();
        let mut buf: [u8; 4] = [0; 4];
        file.read_exact(&mut buf)?;

        let mut table = [0; 256];
        let key = unsafe { std::mem::transmute::<[u8; 4], u32>(buf) } ^ 0x55555555;
        let mut k = key;
        for i in &mut table {
            k = k.rotate_right(1).wrapping_mul(PRIME);
            *i = k;
        }

        Ok(Self {
            file,
            table,
            key,
            buf,
        })
    }

    fn read_int(&mut self) -> Result<u32, std::io::Error> {
        let ret = self.next_int();
        for byte in self.buf {
            self.key ^= self.table[byte as usize];
        }
        ret
    }

    fn next_int(&mut self) -> Result<u32, std::io::Error> {
        self.file.read_exact(&mut self.buf)?;
        let val = unsafe { std::mem::transmute::<[u8; 4], u32>(self.buf) };
        Ok(val ^ self.key)
    }

    #[allow(dead_code)]
    fn next_float(&mut self) -> Result<f32, std::io::Error> {
        Ok(self.next_int()? as f32)
    }

    fn read_byte(&mut self) -> Result<u8, Error> {
        let mut buf = [0; 1];
        self.file.read_exact(&mut buf)?;
        let byte = buf[0];
        self.key ^= self.table[byte as usize];
        Ok((byte as u32 ^ self.key) as u8)
    }

    fn read_bool(&mut self) -> Result<bool, Error> {
        Ok(self.read_byte()? != 0)
    }

    fn read_str(&mut self) -> Result<String, Error> {
        let len = self.read_int()?;
        let mut str_buf: Vec<u8> = vec![0; len.try_into().unwrap()];
        if len > 0 {
            self.file.read_exact(&mut str_buf)?;
            for i in 0..len {
                let byte = (str_buf[i as usize] as u32 ^ self.key) as u8;
                self.key ^= self.table[str_buf[i as usize] as usize];
                str_buf[i as usize] = byte;
            }
            let ret_str = String::from_utf8(str_buf).unwrap();
            return Ok(ret_str);
        }
        Ok("".to_string())
    }

    fn read_block_start(&mut self) -> Result<(u32, Block), Error> {
        let block_start = self.read_int()?;
        let len = self.next_int()?;
        let stream_pos = self.file.stream_position()?;
        let end = u32::try_from(stream_pos).unwrap() + len;
        Ok((block_start, Block { len, end }))
    }

    fn read_block_end(&mut self, block: &Block) -> Result<bool, Error> {
        let stream_pos = u32::try_from(self.file.stream_position()?).unwrap();
        if block.end != stream_pos {
            println!(
                "Stream position is {stream_pos} but block end is {}. Delta: {}",
                block.end,
                stream_pos.abs_diff(block.end)
            );
        }
        if self.next_int()? != 0 {
            println!("Expected end of block character 0.");
            Ok(false)
        } else {
            Ok(true)
        }
    }
}
fn main() -> Result<(), Error> {
    let file_path = std::path::PathBuf::from("transfer.gst");
    let mut decrypt = Decrypt::new(&file_path)?;
    let val = decrypt.read_int()?;
    assert_eq!(val, 2);

    let (block_pos, block) = decrypt.read_block_start()?;
    assert_eq!(block_pos, 18);
    let stash_version = decrypt.read_int()?;
    assert_eq!(stash_version, 5); // Stash file version 5
    assert_eq!(decrypt.next_int()?, 0);
    let _str_mod = decrypt.read_str()?;
    //print!("{str_mod}");

    if stash_version >= 5 {
        let _has_expansion1 = decrypt.read_bool()?; // does this refer to AoM?
        //println!("bool is {has_expansion1}");
    }

    let tabs_count = decrypt.read_int()?;
    let mut tabs = Vec::new();

    for _ in 0..tabs_count {
        let mut items = Vec::new();
        let (_block_start, block) = decrypt.read_block_start()?;
        let _stash_width = decrypt.read_int()?;
        let _stash_height = decrypt.read_int()?;
        let item_count = decrypt.read_int()?;

        for _ in 0..item_count {
            let item = GDItem::read(&mut decrypt)?;
            items.push(item);
        }
        tabs.push(items);
        decrypt.read_block_end(&block)?;
    }
    for tab in tabs {
        for item in tab {
            println!("{:?}", item)
        }
    }
    decrypt.read_block_end(&block)?;

    Ok(())
}
