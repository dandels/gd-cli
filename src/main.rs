mod arc_parser;
mod arz_parser;
mod byte_reader;
mod stash;
mod stash_entry;

use arc_parser::ArcParser;
use arz_parser::*;
use byte_reader::ByteReader;
use stash::Stash;

use std::io::Error;
use std::path::PathBuf;

fn main() -> Result<(), Error> {
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
    //let hand_tag = tags
    //    .tags
    //    .get("records/items/gearhands/d010_hands.dbr")
    //    .unwrap();
    //println!("{hand_tag}");

    let localization_files = [
        "/home/dee/games/Grim Dawn/resources/Text_EN.arc",
        "/home/dee/games/Grim Dawn/gdx1/resources/Text_EN.arc",
        "/home/dee/games/Grim Dawn/gdx2/resources/Text_EN.arc",
    ];

    let mut en_data = ArcParser::new();
    for file in localization_files {
        let path = PathBuf::from(file);
        en_data.add_archive(&path)?;
    }
    //let hands = en_data.map.get(hand_tag).unwrap();
    //println!("{hands}");

    //for record in db.records {
    //    let string = String::from_utf8_lossy(&record.data);
    //    println!("{string}");
    //}
    for (tab_index, tab) in stash.tabs.iter().enumerate() {
        for stash_entry in tab {
            if let Some(EntryType::Item(_record_name, tag_name)) =
                tag_names.items.get(&stash_entry.base_name)
            {
                if let Some(name) = en_data.map.get(tag_name) {
                    let mut prefix: &String = &"".to_string();
                    if !stash_entry.prefix_name.is_empty() {
                        let tag_prefix = tag_names.affixes.get(&stash_entry.prefix_name);
                        if let Some(EntryType::Affix(_, affix_info)) = tag_prefix {
                            if let Some(name) = &affix_info.name {
                                prefix = name;
                            } else if let Some(tag_name) = &affix_info.tag_name {
                                if let Some(name) = en_data.map.get(tag_name) {
                                    prefix = name;
                                    //println!("we have a prefix! {name}");
                                }
                            }
                        }
                    }
                    let mut suffix: &String = &"".to_string();
                    if !stash_entry.suffix_name.is_empty() {
                        let tag_suffix = tag_names.affixes.get(&stash_entry.suffix_name);
                        if let Some(EntryType::Affix(_, affix_info)) = tag_suffix {
                            if let Some(name) = &affix_info.name {
                                suffix = name;
                            } else if let Some(tag_name) = &affix_info.tag_name {
                                if let Some(name) = en_data.map.get(tag_name) {
                                    suffix = name;
                                    //println!("we have a prefix! {name}");
                                }
                            }
                        }
                    }
                    //let name = en_data.map.get(tag_name).unwrap();
                    //println!("found tag {}", tag_name);
                    let tab_nr = tab_index + 1;
                    if prefix.is_empty() {
                        println!("Tab {tab_nr}: {name} {suffix}");
                    } else {
                        println!("Tab {tab_nr}: {prefix} {name} {suffix}");
                    }
                } else {
                    println!("no name for {}", tag_name);
                }
            }
        }
    }
    Ok(())
}
