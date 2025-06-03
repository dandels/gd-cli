mod arc_parser;
mod arz_parser;
mod byte_reader;
mod config;
mod decrypt;
mod inventory_item;
mod item_search;
mod player;
mod stash;

use arc_parser::ArcParser;
use arz_parser::*;
use byte_reader::ByteReader;
use config::Config;
use item_search::ItemLookup;
use player::CharacterItems;
use stash::Stash;

use std::io::Error;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), Error> {
    let mut args = std::env::args();
    let mut search_term = args.nth(1).unwrap_or_default().to_lowercase();
    for arg in args {
        search_term.push_str(&(" ".to_owned() + &arg.to_lowercase()));
    }

    let config = Config::new();

    if config.installation_dir().is_none() {
        println!("The game installation dir needs to be configured.");
        return Ok(());
    }

    if config.save_dir().is_none() {
        println!("The save dir needs to be configured.");
        return Ok(());
    }

    let db_threads_count = Arc::new(AtomicU16::new(0));

    let tag_names = Arc::new(RwLock::new(ArzParser::new()));
    for path in config.get_databases() {
        let tag_names = tag_names.clone();
        db_threads_count.fetch_add(1, Ordering::AcqRel);
        let db_threads_count = db_threads_count.clone();
        thread::spawn(move || {
            tag_names.write().unwrap().add_archive(&path).unwrap();
            db_threads_count.fetch_sub(1, Ordering::AcqRel);
        });
    }

    let localization_data = Arc::new(RwLock::new(ArcParser::new()));
    for path in config.get_localization_files() {
        let localization_data = localization_data.clone();
        db_threads_count.fetch_add(1, Ordering::AcqRel);
        let db_threads_count = db_threads_count.clone();
        thread::spawn(move || {
            localization_data
                .write()
                .unwrap()
                .add_archive(&path)
                .unwrap();
            db_threads_count.fetch_sub(1, Ordering::AcqRel);
        });
    }

    let lookup = Arc::new(ItemLookup {
        search_term,
        localization_data,
        tag_names,
    });

    let all_char_items = Arc::new(RwLock::new(Vec::new()));
    for save in config.get_save_files() {
        let all_char_items = all_char_items.clone();
        thread::spawn(move || match CharacterItems::read(&save) {
            Ok(ci) => {
                all_char_items.write().unwrap().push(ci);
            }
            Err(e) => {
                println!("Unable to read save file {:?}: {e}", save);
            }
        });
    }

    let search_threads_count = Arc::new(AtomicU16::new(0));

    while db_threads_count.load(Ordering::SeqCst) > 0 {
        sleep(Duration::from_millis(5));
    }

    for stash_path in config.get_stash_files() {
        search_threads_count.fetch_add(1, Ordering::AcqRel);
        let lookup = lookup.clone();
        let search_thread_count = search_threads_count.clone();
        thread::spawn(move || {
            let stash = Stash::new(&stash_path).unwrap();
            for (i, tab) in stash.tabs.iter().enumerate() {
                for inventory_item in tab {
                    lookup.check_item(inventory_item, &format!("Shared stash tab {}", i + 1));
                }
            }
            search_thread_count.fetch_sub(1, Ordering::AcqRel);
        });
    }

    for char_items in Arc::into_inner(all_char_items)
        .unwrap()
        .into_inner()
        .unwrap()
    {
        let lookup = lookup.clone();
        search_threads_count.fetch_add(1, Ordering::AcqRel);
        let search_thread_count = search_threads_count.clone();
        thread::spawn(move || {
            for (i, bag) in char_items.inventory.bags.iter().enumerate() {
                for inventory_item in &bag.items {
                    lookup.check_item(
                        inventory_item,
                        &format!("{} bag {}", char_items.name, i + 1),
                    );
                }
            }

            for inventory_item in char_items.inventory.equipment.iter() {
                lookup.check_item(
                    &inventory_item.item,
                    &format!("Equipped by {}", char_items.name),
                );
            }
            search_thread_count.fetch_sub(1, Ordering::AcqRel);
        });
    }

    // Wait for threads to finish
    loop {
        std::thread::sleep(Duration::from_millis(50));
        if search_threads_count.load(Ordering::SeqCst) == 0 {
            break;
        }
    }
    Ok(())
}
