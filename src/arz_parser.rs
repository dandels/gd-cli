use crate::byte_reader::ByteReader;
use std::collections::HashMap;
use std::path::PathBuf;
use std::io::Error;
use std::thread;
use std::sync::mpsc;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ArzRecordHeader {
    string_index: u32,
    record_type: String,
    offset: u32,
    size_compressed: u32,
    size_decompressed: u32,
}

impl ArzRecordHeader {
    fn read(byte_vec: &mut ByteReader) -> Self {
        let string_index = byte_vec.read_u32();
        let str_len = byte_vec.read_u32();
        let record_type = byte_vec.read_string(str_len);
        Self {
            string_index, record_type,
            offset: byte_vec.read_u32(),
            size_compressed: byte_vec.read_u32(),
            size_decompressed: byte_vec.read_u32(),
        }
    }
}

// v3 of the header?
struct ArzArchiveHeader {
    unknown: u16, // Item Assistant code thinks this is the version check?
    version: u16,
    records_start: u32,
    #[allow(dead_code)]
    records_len: u32,
    records_count: u32,
    strings_start: u32,
    strings_size: u32,
}

impl ArzArchiveHeader {
    fn new(byte_vec: &mut ByteReader) -> Self {
        Self {
            unknown: byte_vec.read_u16(),
            version: byte_vec.read_u16(),
            records_start: byte_vec.read_u32(),
            records_len: byte_vec.read_u32(),
            records_count: byte_vec.read_u32(),
            strings_start: byte_vec.read_u32(),
            strings_size: byte_vec.read_u32(),
        }
    }
}

#[derive(Debug)]
struct EntryHeader {
    entry_type: u16,
    entry_count: u16,
    string_index: u32,
}

impl EntryHeader {
    fn read(byte_vec: &mut ByteReader) -> Self {
        Self {
            entry_type: byte_vec.read_u16(),
            entry_count: byte_vec.read_u16(),
            string_index: byte_vec.read_u32(),
        }
    }
}

type Items = HashMap<String, EntryType>;
type Affixes = HashMap<String, EntryType>;

pub fn read_archive(path: &PathBuf) -> Result<(Items, Affixes), Error> {
    let mut reader = ByteReader::from_file(path)?;

    let archive_header = ArzArchiveHeader::new(&mut reader);

    // Asserts copied from Item Assistant example
    assert_eq!(archive_header.unknown, 2);
    assert_eq!(archive_header.version, 3);

    let strings = Arc::new(read_strings(&mut reader, &archive_header));
    let record_headers = read_record_headers(&mut reader, &archive_header);

    let (tx, rx) = mpsc::channel();
    let mut threads = 0;
    let mut thread_names = Vec::new();

    'header_loop: for record_header in record_headers {
        let record_name = strings[record_header.string_index as usize].clone();
        // Uncomment to debug why something is not getting properly read
        // note for debugging: record_type.is_empty() also yields values
        //let catch = "records/items/crafting/blueprints/other/craft_potion_royaljellyointment.dbr";
        //if record_name == catch {
        //    println!("{record_name}: {:?}", record_header.record_type);
        //}

        if
            record_header.record_type.starts_with("Armor") 
            || record_header.record_type.starts_with("Item") 
            || record_header.record_type.starts_with("QuestItem")
            || record_header.record_type.starts_with("Weapon") 
            || record_header.record_type.starts_with("OneShot_Scroll") 
            // starts_with() would also match "LootRandomizerTable"
            || record_header.record_type == "LootRandomizer"
        {
            if record_header.record_type.starts_with("Item") {
                let ignore_list = [
                    "ItemTransmuter",
                    "ItemTransmuterSet",
                    "ItemSetFormula",
                    "ItemRandomSetFormula",
                ];
                for ign in ignore_list {
                    if record_header.record_type.starts_with(ign) {
                        continue 'header_loop;
                    }
                }

                //println!("{}", record_header.record_type);
            }
            if record_name.starts_with("records/items/") 
                || record_name.starts_with("records/creatures/npcs/npcgear/")
                || record_name.starts_with("records/storyelements/")
                || record_name.starts_with("records/endlessdungeon/") {
                //println!("record type {}", record_header.record_type);
                let ignore_list = [
                    "records/items/enemygear/",
                    "records/items/transmutes/",
                    // Searching for unique affixes. Maybe later.
                    "records/items/lootaffixes/prefixunique/", 
                    "records/items/lootaffixes/suffixunique/",
                    "records/items/lootaffixes/completionrelics",
                    "records/items/lootaffixes/completion",
                    "records/items/lootaffixes/crafting",
                ];
                for ign in ignore_list {
                    if record_name.starts_with(ign) {
                        continue 'header_loop;
                    }
                }

                threads += 1;
                thread_names.push(record_name.clone());
                let strings = strings.clone();
                let mut reader = reader.clone();

                let tx = tx.clone();
                // TODO this spawns needlessly many threads
                thread::spawn(move || {
                    let data = decompress(&mut reader, &record_header);
                    let is_affix = record_header.record_type == "LootRandomizer";
                    let entry = parse_record(&record_header, data, &record_name, &strings, is_affix);
                    tx.send(Some((record_name, entry, is_affix))).unwrap();
                });
            }
        }
    }

    let mut items = Items::new();
    let mut affixes = Affixes::new();

    #[allow(clippy::needless_range_loop)]
    for i in 0..threads {
        match rx.recv() {
            Ok(msg) => {
                if let Some((record_name, entry, is_affix)) = msg {
                    match entry {
                        Some(e) => {
                            if is_affix {
                                affixes.insert(record_name, e);
                            } else {
                                items.insert(record_name, e);
                            }
                        }
                        None => {
                            println!("nothing found for {record_name}");
                        }
                    }
                }
            }
            Err(e) => {
                println!("recv for {} failed with err {e}", &thread_names[i]);
            }
        }
    }
    Ok((items, affixes))
}

// Used by the logic in parse_record(). Knowing the type of the record could be important later.
#[derive(Debug)]
#[allow(dead_code)] 
enum EntryValue {
    Float(f32),
    Text(String),
    Int(u32),
}

#[derive(Debug)]
pub enum EntryType {
    Affix(AffixInfo),
    Item(String, String, String, Option<u32>), // record name, tag name, rarity, level req
}

#[derive(Debug)]
pub struct AffixInfo {
    pub tag_name: Option<String>,
    pub rarity: String, // the affixes could be printed in color with this
    pub name: Option<String>
}

fn parse_record(record_header: &ArzRecordHeader, data: Vec<u8>, record_name: &str, strings: &[String], is_affix: bool) -> Option<EntryType> {
    let mut reader = ByteReader::from_vec(data);

    let mut vals: Vec<(String, EntryValue)> = Vec::new();
    let mut tag_name: Option<String> = None; // used by most items and affixes
    let mut description: Option<String> = None; // fallback for relics that don't have itemNameTag
    let mut rarity: Option<String> = None;
    let mut level_req: Option<u32> = None;

    //println!("Processing record: {record_name}");

    let mut i = 0;
    'outer: while i < record_header.size_decompressed / 4 {
        let entry_header = EntryHeader::read(&mut reader);
        i += 2 + entry_header.entry_count as u32;
        let entry_key = &strings[entry_header.string_index as usize];
        //println!("entry key {entry_key}");
        for _ in 0..entry_header.entry_count {
            let entry_value = match entry_header.entry_type {
                1 => EntryValue::Float(reader.read_f32()),
                2 => {
                    let int = reader.read_u32();
                    let value = &strings[int as usize];
                    match entry_key.as_str() {
                        "lootRandomizerName" | "itemNameTag" => { tag_name = Some(value.clone()); }
                        "itemClassification" => { rarity = Some(value.clone()); }
                        "description" => { description = Some(value.clone()); }
                        _ => {}
                    }
                    EntryValue::Text(value.clone())
                },
                _ =>
                { 

                    let int = reader.read_u32();
                    //Seems like the "levelRequirement" field isn't useful..?
                    if entry_key.as_str() == "itemLevel" {
                        level_req = Some(int);
                    }
                    EntryValue::Int(int) 
                },
            };

            // Stop reading data once we found what we came for.
            // We only need these fields for items
            if !is_affix && tag_name.is_some() && level_req.is_some() && rarity.is_some() {
                break 'outer;
            }
            // We can also use the rarity for affixes to display them nicely in the UI
            if tag_name.is_some() && rarity.is_some() {
                break 'outer;
            }

            // These are actually only used when debugging
            vals.push((entry_key.clone(), entry_value));
        }
    }
    let rarity = rarity.unwrap_or_default();
    if is_affix {
        if tag_name.is_none() {
            //println!("Nothing found for: {:?}", record_name);
            //println!("{:?}", vals);
        }
        let ai = AffixInfo { tag_name, rarity, name: None };
        Some(EntryType::Affix(ai))
    } else {
        //println!("{}, {record_name} {:?}", record.header.record_type, tag_name);
        #[allow(clippy::manual_map)]
        if let Some(name) = tag_name {
            return Some(EntryType::Item(record_name.to_string(), name.clone(), rarity, level_req))
        } else if let Some(desc) = description {
            if !desc.is_empty() {
                //println!("No tag but had description: {}, {record_name} {:?}", record_header.record_type, tag_name);
                return Some(EntryType::Item(record_name.to_string(), desc.clone(), rarity, level_req))
            } else {
                println!("Empty tag and description: {}, {record_name} {:?}", record_header.record_type, tag_name);
            }
        }
        // Uncomment to debug what is getting parsed
        //println!("No tagname found for {record_name}.", );
        //for (key, val) in vals {
        //    println!("{key}: {:?}", val);
        //}
        // we tried everything, so maybe use record_name as tag
        Some(EntryType::Item(record_name.to_string(), record_name.to_string(), rarity, level_req))
    }
}

fn decompress(byte_vec: &mut ByteReader, header: &ArzRecordHeader) -> Vec<u8> {
    byte_vec.index = header.offset as usize + 24;
    let end = byte_vec.index + header.size_compressed as usize;
    let slice = &byte_vec.bytes[byte_vec.index..end];
    lz4::block::decompress(slice, Some(header.size_decompressed.try_into().unwrap())).unwrap()
}

fn read_record_headers(byte_vec: &mut ByteReader, header: &ArzArchiveHeader) -> Vec<ArzRecordHeader> {
    let mut records = Vec::new();
    byte_vec.index = header.records_start as usize;
    for _ in 0..header.records_count {
        let record = ArzRecordHeader::read(byte_vec);
        records.push(record);
        byte_vec.index += 8;
    }
    records
}

fn read_strings(byte_vec: &mut ByteReader, header: &ArzArchiveHeader) -> Vec<String> {
    let mut strings = Vec::new();
    byte_vec.index = (header.strings_start) as usize;
    let end = (header.strings_start + header.strings_size) as usize;
    while byte_vec.index < end {
        let count = byte_vec.read_u32();
        for _ in 0..count {
            let len = byte_vec.read_u32();
            let string = byte_vec.read_string(len);
            strings.push(string);
        }
    }
    strings
}
