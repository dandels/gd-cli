mod arc_parser;
mod arz_parser;
mod byte_reader;
mod decrypt;
mod player;
mod stash;
mod inventory_item;

use arc_parser::ArcParser;
use arz_parser::*;
use byte_reader::ByteReader;
use player::CharacterItems;
use stash::Stash;
use inventory_item::InventoryItem;

use std::io::Error;
use std::path::PathBuf;

struct ItemLookup {
    localization_data: ArcParser,
    tag_names: ArzParser,
}

struct CompleteItem {
    name: String,
    prefix: Option<String>,
    suffix: Option<String>,
}

impl ItemLookup {
    fn lookup_item(&self, inventory_item: &InventoryItem) -> Option<CompleteItem> {
        if let Some(EntryType::Item(_record_name, tag_name)) = self.tag_names.items.get(&inventory_item.base_name) {
            if let Some(name) = self.localization_data.map.get(tag_name) {
                let mut prefix = None;
                if !inventory_item.prefix_name.is_empty() {
                    let tag_prefix = self.tag_names.affixes.get(&inventory_item.prefix_name);
                    if let Some(EntryType::Affix(affix_info)) = tag_prefix {
                        if let Some(name) = &affix_info.name {
                            prefix = Some(name.clone());
                        } else if let Some(tag_name) = &affix_info.tag_name {
                            if let Some(name) = self.localization_data.map.get(tag_name) {
                                prefix = Some(name.clone());
                            }
                        }
                    }
                }
                let mut suffix = None;
                if !inventory_item.suffix_name.is_empty() {
                    let tag_suffix = self.tag_names.affixes.get(&inventory_item.suffix_name);
                    if let Some(EntryType::Affix(affix_info)) = tag_suffix {
                        if let Some(name) = &affix_info.name {
                            suffix = Some(name.clone());
                        } else if let Some(tag_name) = &affix_info.tag_name {
                            if let Some(name) = self.localization_data.map.get(tag_name) {
                                suffix = Some(name.clone());
                            }
                        }
                    }
                }
                Some(CompleteItem { name: name.clone(), prefix, suffix })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn iterate_item(&self, inventory_item: &InventoryItem, msg_prefix: &str) {
        if let Some(ci) = self.lookup_item(inventory_item) {
            if ci.prefix.is_none() {
                println!("{msg_prefix}{} {}", ci.name, ci.suffix.unwrap_or_default());
            } else {
                println!("{msg_prefix}{} {} {}", ci.prefix.unwrap(), ci.name, ci.suffix.unwrap_or_default());
            }
        } else {
            println!("No tag found for {}", inventory_item.base_name);
        }

    }

}

fn main() -> Result<(), Error> {
    let save_path = std::path::PathBuf::from("/home/dee/src/rust/gd-cli/player.gdc");
    let char_items = CharacterItems::read(&save_path)?;
    let stash_path = std::path::PathBuf::from("/home/dee/src/rust/gd-cli/transfer.gst");
    let stash = Stash::new(&stash_path)?;
    //println!("{:?}", stash.tabs);

    //println!("{:?}", en_data.map);

    let db_files = [
        "/home/dee/games/Grim Dawn/database/database.arz",
        "/home/dee/games/Grim Dawn/gdx1/database/GDX1.arz",
        "/home/dee/games/Grim Dawn/gdx2/database/GDX2.arz",
    ];
    //let db_path = PathBuf::from("/home/dee/games/Grim Dawn/database/database.arz");
    let mut tag_names = ArzParser::new();
    for file in db_files {
        let path = PathBuf::from(file);
        tag_names.add_archive(&path)?;
    }

    let localization_files = [
        "/home/dee/games/Grim Dawn/resources/Text_EN.arc",
        "/home/dee/games/Grim Dawn/gdx1/resources/Text_EN.arc",
        "/home/dee/games/Grim Dawn/gdx2/resources/Text_EN.arc",
    ];

    let mut localization_data = ArcParser::new();
    for file in localization_files {
        let path = PathBuf::from(file);
        localization_data.add_archive(&path)?;
    }

    let lookup = ItemLookup { localization_data, tag_names };

    for (i, tab) in stash.tabs.iter().enumerate() {
        for inventory_item in tab {
            lookup.iterate_item(inventory_item, &format!("Shared stash tab {}: ", i + 1));
        }
    }

    for (i, bag) in char_items.inventory.bags.iter().enumerate() {
        for inventory_item in &bag.items {
            lookup.iterate_item(inventory_item, &format!("{} bag {}: ", char_items.name, i + 1));
        }
    }

    for (i, inventory_item) in char_items.inventory.equipment.iter().enumerate() {
        lookup.iterate_item(&inventory_item.item, &format!("{} item {}: ", char_items.name, i + 1));
    }

    Ok(())
}
