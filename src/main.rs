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

use std::{fmt, fmt::Display};
use std::io::Error;
use std::path::PathBuf;
use std::thread;
use std::sync::Arc;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::sync::RwLock;
use std::thread::sleep;

struct ItemLookup {
    search_term: String,
    localization_data: Arc<RwLock<ArcParser>>,
    tag_names: Arc<RwLock<ArzParser>>,
}

struct CompleteItem {
    name: String,
    prefix: Option<String>,
    suffix: Option<String>,
}

impl Display for CompleteItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.prefix.is_none() {
            write!(f, "{} {}", self.name, self.suffix.as_ref().unwrap_or(&"".to_string()))
        } else {
            write!(f, "{} {} {}", self.prefix.as_ref().unwrap(), self.name, self.suffix.as_ref().unwrap_or(&"".to_string()))
        }
    }
}

impl ItemLookup {
    fn lookup_item(&self, inventory_item: &InventoryItem) -> Option<CompleteItem> {
        let tag_names = self.tag_names.read().unwrap();
        let localization_data = self.localization_data.read().unwrap();
        if let Some(EntryType::Item(_record_name, tag_name)) = tag_names.items.get(&inventory_item.base_name) {
            if let Some(name) = localization_data.map.get(tag_name) {
                let mut prefix = None;
                if !inventory_item.prefix_name.is_empty() {
                    let tag_prefix = tag_names.affixes.get(&inventory_item.prefix_name);
                    if let Some(EntryType::Affix(affix_info)) = tag_prefix {
                        if let Some(name) = &affix_info.name {
                            prefix = Some(name.clone());
                        } else if let Some(tag_name) = &affix_info.tag_name {
                            if let Some(name) = localization_data.map.get(tag_name) {
                                prefix = Some(name.clone());
                            }
                        }
                    }
                }
                let mut suffix = None;
                if !inventory_item.suffix_name.is_empty() {
                    let tag_suffix = tag_names.affixes.get(&inventory_item.suffix_name);
                    if let Some(EntryType::Affix(affix_info)) = tag_suffix {
                        if let Some(name) = &affix_info.name {
                            suffix = Some(name.clone());
                        } else if let Some(tag_name) = &affix_info.tag_name {
                            if let Some(name) = localization_data.map.get(tag_name) {
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

    fn check_item(&self, inventory_item: &InventoryItem, item_source: &str) {
        if let Some(ci) = self.lookup_item(inventory_item) {
            let item_name = ci.to_string();
            if item_name.to_lowercase().contains(&self.search_term) {
                println!("{item_source}: {item_name}");
            }
        } else {
            //println!("No tag found for {}", inventory_item.base_name);
        }
    }
}

fn main() -> Result<(), Error> {
    let mut args = std::env::args();
    let mut search_term = args.nth(1).unwrap_or_default().to_lowercase();
    for arg in args {
        search_term.push_str(&(" ".to_owned() + &arg.to_lowercase()));
     }

    let db_threads_count = Arc::new(AtomicU16::new(0));
    let db_files = [
        "/home/dee/games/Grim Dawn/database/database.arz",
        "/home/dee/games/Grim Dawn/gdx1/database/GDX1.arz",
        "/home/dee/games/Grim Dawn/gdx2/database/GDX2.arz",
    ];

    let tag_names = Arc::new(RwLock::new(ArzParser::new()));
    for file in db_files {
        let path = PathBuf::from(file);
        let tag_names = tag_names.clone();
        db_threads_count.fetch_add(1, Ordering::AcqRel);
        let db_threads_count = db_threads_count.clone();
        thread::spawn(move || {
            tag_names.write().unwrap().add_archive(&path).unwrap();
            db_threads_count.fetch_sub(1, Ordering::AcqRel);
        });
    }

    let localization_files = [
        "/home/dee/games/Grim Dawn/resources/Text_EN.arc",
        "/home/dee/games/Grim Dawn/gdx1/resources/Text_EN.arc",
        "/home/dee/games/Grim Dawn/gdx2/resources/Text_EN.arc",
    ];

    let localization_data = Arc::new(RwLock::new(ArcParser::new()));
    for file in localization_files {
        let path = PathBuf::from(file);
        let localization_data = localization_data.clone();
        db_threads_count.fetch_add(1, Ordering::AcqRel);
        let db_threads_count = db_threads_count.clone();
        thread::spawn(move || {
            localization_data.write().unwrap().add_archive(&path).unwrap();
            db_threads_count.fetch_sub(1, Ordering::AcqRel);
        });
    }

    let lookup = Arc::new(ItemLookup { search_term, localization_data, tag_names });

    let save_path = std::path::PathBuf::from("/home/dee/src/rust/gd-cli/player.gdc");
    let char_items = CharacterItems::read(&save_path)?;
    let thread_count = Arc::new(AtomicU16::new(0));
    {
        let lookup = lookup.clone();
        let thread_count = thread_count.clone();
        let db_threads_count_count = db_threads_count.clone();
        thread::spawn(move || {
            let stash_path = std::path::PathBuf::from("/home/dee/src/rust/gd-cli/transfer.gst");
            let stash = Stash::new(&stash_path).unwrap();
            while db_threads_count_count.load(Ordering::SeqCst) > 0 {
                sleep(Duration::from_millis(5));
            }
            for (i, tab) in stash.tabs.iter().enumerate() {
                for inventory_item in tab {
                    lookup.check_item(inventory_item, &format!("Shared stash tab {}", i + 1));
                }
            }
            thread_count.fetch_sub(1, Ordering::AcqRel);
        });
    }
    thread_count.fetch_add(1, Ordering::AcqRel);

    {
        let lookup = lookup.clone();
        let thread_count = thread_count.clone();
        let name = char_items.name.clone();
        thread::spawn(move || {
            while db_threads_count.load(Ordering::SeqCst) > 0 {
                sleep(Duration::from_millis(5));
            }
            for (i, bag) in char_items.inventory.bags.iter().enumerate() {
                for inventory_item in &bag.items {
                    lookup.check_item(inventory_item, &format!("{} bag {}", name, i + 1));
                }
            }

            for inventory_item in char_items.inventory.equipment.iter() {
                lookup.check_item(&inventory_item.item, &format!("Equipped by {}", char_items.name));
            }
            thread_count.fetch_sub(1, Ordering::AcqRel);
        });
    }
    thread_count.fetch_add(1, Ordering::AcqRel);

    loop {
        std::thread::sleep(Duration::from_millis(50));
        if thread_count.load(Ordering::SeqCst) == 0 {
            break;
        }
    }
    Ok(())
}
