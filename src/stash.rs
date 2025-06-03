use super::inventory_item::InventoryItem;
use super::decrypt::Decrypt;
use std::io::Error;
use std::path::PathBuf;

pub struct StashItem {
    pub item: InventoryItem,
    #[allow(dead_code)]
    x_offset: u32,
    #[allow(dead_code)]
    y_offset: u32,
}

impl StashItem {
    pub fn read(decrypt: &mut Decrypt) -> Result<Self, Error> {
        Ok(Self {
            item: InventoryItem::read(decrypt)?,
            x_offset: decrypt.read_int(),
            y_offset: decrypt.read_int(),
        })
    }
}

pub struct Stash {
    pub tabs: Vec<Vec<InventoryItem>>,
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
                let item = StashItem::read(&mut decrypt)?;
                items.push(item.item);
            }
            tabs.push(items);
            decrypt.read_block_end(&block).unwrap();
        }
        decrypt.read_block_end(&block).unwrap();

        Ok(Self { tabs })
    }
}
