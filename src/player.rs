use crate::inventory_item::InventoryItem;
use crate::stash;
use crate::stash::StashItem;

use super::decrypt::Decrypt;

use std::io::Error;
use std::path::PathBuf;

const EQUIPMENT_SLOTS: usize = 12;

pub struct PlayerStash {
    pub tabs: Vec<Vec<InventoryItem>>
}
impl PlayerStash {
    fn read(decrypt: &mut Decrypt) -> Result<PlayerStash, Error> {
        let (start, block) = decrypt.read_block_start();
        assert!(start == 4, "Expected player stash block to start with 0.");
        assert!(decrypt.read_int() == 6, "Expected character stash version to be 6.");
        let num_tabs = decrypt.read_int();
        let mut tabs = Vec::with_capacity(num_tabs as usize);
        for _ in 0..num_tabs {
            tabs.push(stash::read_stash_tab(decrypt)?);
        }
        decrypt.read_block_end(&block).unwrap();
        Ok(PlayerStash { 
            tabs
        })
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Inventory {
    num_bags: u32,
    pub bags: Vec<Bag>,
    pub equipment: [InventoryEquipment; EQUIPMENT_SLOTS],
    pub weapon_set_1: [InventoryEquipment; 2],
    pub weapon_set_2: [InventoryEquipment; 2],
    focused: u32,
    selected: u32,
    flag: u8,
    use_alternate: u8,
    alternate_1: u8,
    alternate_2: u8,
}

#[derive(Debug)]
pub struct InventoryEquipment {
    pub item: InventoryItem,
    #[allow(dead_code)]
    attached: u8,
}

impl InventoryEquipment {
    fn read(decrypt: &mut Decrypt) -> Self {
        Self {
            item: InventoryItem::read(decrypt).unwrap(),
            attached: decrypt.read_byte(),
        }
    }
}

#[derive(Debug)]
pub struct Bag {
    _some_bool: u8,
    pub items: Vec<InventoryItem>,
}

impl Bag {
    fn read(decrypt: &mut Decrypt) -> Self {
        let (start, block) = decrypt.read_block_start();
        assert!(start == 0, "expected non-zero start of bag block");
        let ret = Self {
            _some_bool: decrypt.read_byte(),
            items: {
                let len = decrypt.read_int();
                let mut ret = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    ret.push(StashItem::read(decrypt).unwrap().item);
                }
                ret
            },
        };
        decrypt.read_block_end(&block).unwrap();
        ret
    }
}

impl Inventory {
    fn read(decrypt: &mut Decrypt) -> Self {
        let (start, block) = decrypt.read_block_start();
        assert_eq!(start, 3);
        assert_eq!(decrypt.read_int(), 4);
        let flag = decrypt.read_byte();
        if flag == 0 {
            panic!("This byte was supposed to be 0. The file format will be wrong and I can't continue.");
        }
        let num_bags = decrypt.read_int();
        let focused = decrypt.read_int();
        let selected = decrypt.read_int();
        let mut bags = Vec::with_capacity(num_bags as usize);
        for _ in 0..num_bags {
            bags.push(Bag::read(decrypt));
        }
        let use_alternate = decrypt.read_byte();
        let equipment = std::array::from_fn(|_| InventoryEquipment::read(decrypt));
        let alternate_1 = decrypt.read_byte();
        let weapon_set_1 = std::array::from_fn(|_| InventoryEquipment::read(decrypt));
        let alternate_2 = decrypt.read_byte();
        let weapon_set_2 = std::array::from_fn(|_| InventoryEquipment::read(decrypt));

        let ret = Self {
            num_bags,
            bags,
            equipment,
            weapon_set_1,
            weapon_set_2,
            focused,
            selected,
            flag,
            use_alternate,
            alternate_1,
            alternate_2,
        };
        decrypt.read_block_end(&block).unwrap();
        ret
    }
}

#[derive(Debug)]
struct PlayerHeader {
    name: String,
    _sex: bool, // which is which?
    _class_tag: String,
    _level: u32,
    _hardcore: bool, // reversed?
}

// this is here just so it's decrypted correctly, simply reading sizeof() bytes didn't work somewhy
#[allow(dead_code)]
struct CharacterInfo {
    is_in_main_quest: u8,
    has_been_in_game: u8,
    difficulty: u8,
    greatest_difficulty: u8,
    money: u32,
    greatest_survival_difficulty: u8,
    current_tribute: u32,
    compass_state: u8,
    skill_window_show_help: u8,
    weapon_swap_active: u8,
    weapon_swap_enabled: u8,
    texture: String,
    loot_filter_len: u32,
    loot_filter: [u8; 39],
}

fn skip_block_with_size_n(decrypt: &mut Decrypt, expected_start: u32, version: u32, size: usize) {
    let (start, block) = decrypt.read_block_start();
    assert_eq!(start, expected_start);
    assert_eq!(decrypt.read_int(), version);
    for _ in 0..size {
        decrypt.read_byte();
    }
    decrypt.read_block_end(&block).unwrap();
}

impl CharacterInfo {
    fn read(decrypt: &mut Decrypt) -> Self {
        let (start, block) = decrypt.read_block_start();
        assert_eq!(start, 1);
        assert_eq!(decrypt.read_int(), 5); // version == 5
        //let texture = decrypt.read_str();
        //let _skipped_ints = size_of::<u32>() * 41;
        //for _ in 0..size_of::<CharacterInfo>() {
        //    decrypt.read_byte();
        //}

        let ret = Self {
            is_in_main_quest: decrypt.read_byte(),
            has_been_in_game: decrypt.read_byte(),
            difficulty: decrypt.read_byte(),
            greatest_difficulty: decrypt.read_byte(),
            money: decrypt.read_int(),
            greatest_survival_difficulty: decrypt.read_byte(),
            current_tribute: decrypt.read_int(),
            compass_state: decrypt.read_byte(),
            skill_window_show_help: decrypt.read_byte(),
            weapon_swap_active: decrypt.read_byte(),
            weapon_swap_enabled: decrypt.read_byte(),
            texture: decrypt.read_str().unwrap(),
            loot_filter_len: decrypt.read_int(),
            loot_filter: {
                let mut buf = [0; 39];
                for byte in buf.iter_mut() {
                    *byte = decrypt.read_byte();
                }
                buf
            },
        };
        decrypt.read_block_end(&block).unwrap();
        ret
    }
}

impl PlayerHeader {
    fn read(decrypt: &mut Decrypt) -> Self {
        Self {
            name: decrypt.read_wide_string().unwrap(),
            _sex: decrypt.read_bool(),
            _class_tag: decrypt.read_str().unwrap(),
            _level: decrypt.read_int(),
            _hardcore: decrypt.read_bool(),
        }
    }
}

pub struct CharacterItems {
    pub name: String,
    pub inventory: Inventory,
    pub stash: PlayerStash,
}

impl CharacterItems {
    pub fn read(path: &PathBuf) -> Result<Self, Error> {
        let mut decrypt = Decrypt::new(path)?;
        assert_eq!(decrypt.read_int(), 0x58434447);
        assert_eq!(decrypt.read_int(), 2);
        let header = PlayerHeader::read(&mut decrypt);
        let _byte = decrypt.read_byte();
        assert_eq!(decrypt.next_int(), 0); // end of block
        assert_eq!(decrypt.read_int(), 8); // version

        let mut uid_buf: [u8; 16] = [0; 16];
        for byte in uid_buf.iter_mut() {
            *byte = decrypt.read_byte();
        }
        let _char_info = CharacterInfo::read(&mut decrypt);
        //skip_character_bio(&mut decrypt);
        skip_block_with_size_n(&mut decrypt, 2, 8, 44); // skip bio
        let inventory = Inventory::read(&mut decrypt);
        let stash = PlayerStash::read(&mut decrypt)?;

        Ok(Self {
            name: header.name,
            inventory,
            stash,
        })
    }
}
