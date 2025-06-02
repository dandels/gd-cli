use crate::byte_reader::ByteReader;
use std::collections::HashMap;
use std::path::PathBuf;
use std::io::Error;

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

pub struct ArzRecord {
    pub header: ArzRecordHeader,
    pub data: Vec<u8>,
}

// v3 of the header?
struct ArzArchiveHeader {
    unknown: u16, // Item Assistant code thinks this is the version check?
    version: u16,
    records_start: u32,
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

#[derive(Debug)]
pub struct ArzParser {
    //pub records: Vec<ArzRecord>
    pub items: HashMap<String, EntryType>,
    pub affixes: HashMap<String, EntryType>
}

impl ArzParser {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            affixes: HashMap::new()
        }
    }

    pub fn add_archive(&mut self, path: &PathBuf) -> Result<(), Error> {
        let mut reader = ByteReader::from_file(path)?;

        let archive_header = ArzArchiveHeader::new(&mut reader);

        // Asserts copied from IA example
        assert_eq!(archive_header.unknown, 2);
        assert_eq!(archive_header.version, 3);

        let strings = read_strings(&mut reader, &archive_header);
        let record_headers = read_record_headers(&mut reader, &archive_header);

        for record_header in &record_headers {
            if
                record_header.record_type.starts_with("Armor") 
                    || record_header.record_type.starts_with("Item") 
                    || record_header.record_type.starts_with("Weapon") 
                    || record_header.record_type == "LootRandomizer" 
                  //|| record_header.record_type.is_empty()
            {
                if record_header.record_type.starts_with("Item") {
                    //println!("{}", record_header.record_type);
                    continue;
                }
                let record_name = strings[record_header.string_index as usize].clone();
                if record_name.starts_with("records/items/") {
                    //println!("record type {}", record_header.record_type);
                    if record_name.starts_with("records/items/enemygear/") {
                        continue;
                    }
                    let record = ArzRecord {
                        header: record_header.clone(),
                        data: decompress(&mut reader, record_header)
                    };
                    let is_affix = record_header.record_type == "LootRandomizer";
                    let entry = parse_record(&record, &record_name, &strings, is_affix);
                    if is_affix {
                        self.affixes.insert(record_name, entry);
                    } else {
                        self.items.insert(record_name, entry);
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct RecordEntry {
    record_name: String,
    tag_name: Option<String>,
    values: Vec<EntryValue>
}

impl RecordEntry {
    fn new(record_name: String) -> Self {
        Self {
            record_name,
            tag_name: None,
            values: Vec::new()
        }
    }
}

#[derive(Debug)]
enum EntryValue {
    Float(f32),
    Text(String),
    Int(u32),
}

#[derive(Debug)]
pub enum EntryType {
    Affix(String, AffixInfo), // String is record name
    Item(String, String), // record name, tag name
}

#[derive(Debug)]
pub struct AffixInfo {
    pub tag_name: Option<String>,
    pub rarity: String,
    pub name: Option<String>
}

fn parse_record(record: &ArzRecord, record_name: &str, strings: &[String], is_affix: bool) -> EntryType {
    let mut reader = ByteReader::from_slice(&record.data);

    //let mut record_entry = RecordEntry::new(record_name.clone());

    let mut vals: Vec<(String, EntryValue)> = Vec::new();
    let mut tag_name: Option<String> = None;
    let mut rarity: Option<String> = None;

    //println!("Processing record: {record_name}");

    let mut i = 0;
    'outer: while i < record.header.size_decompressed / 4 {
        let entry_header = EntryHeader::read(&mut reader);
        i += 2 + entry_header.entry_count as u32;
        let entry_key = &strings[entry_header.string_index as usize];
        for _ in 0..entry_header.entry_count {
            let entry_value = match entry_header.entry_type {
                1 => EntryValue::Float(reader.read_f32()),
                2 => {
                    let int = reader.read_u32();
                    let value = &strings[int as usize];
                    if entry_key == "lootRandomizerName" || entry_key == "itemNameTag" {
                       tag_name = Some(value.clone());
                    } else if entry_key == "itemClassification" {
                        rarity = Some(value.clone());
                    }
                    EntryValue::Text(value.clone())
                },
                _ => EntryValue::Int(reader.read_u32()),
            };

            // Stop reading data once we found what we came for.
            // We only need the tag name for items
            if !is_affix && tag_name.is_some() {
                //println!("job's done");
                break 'outer;
            }
            // We can also use the rarity for affixes to display them nicely in the UI
            if tag_name.is_some() && rarity.is_some() {
                break 'outer;
            }

            //println!("{entry_key}: {:?}", entry_value);
            //println!("{record_name} - {:?}", stat);
            vals.push((entry_key.clone(), entry_value));
            //if record_string == "itemNameTag" {
            //    if let ItemStat::IntField(_, index) = stat {
            //        let tag_name = strings[index as usize].clone();
            //        return Some((record_name, tag_name));
            //    }
            //}
            //if record_string == "lootName" {
            //    println!("lootName {record_string}");
            //}
        }
    }
    if is_affix {
        if tag_name.is_none() {
            //println!("No tag found for: {:?}", record_name);
            //println!("{:?}", vals);
        }
        let ai = AffixInfo { tag_name, rarity: rarity.unwrap(), name: None };
        EntryType::Affix(record_name.to_string(), ai)
    } else {
        //println!("{}, {record_name} {:?}", record.header.record_type, tag_name);
        if let Some(name) = tag_name {
            EntryType::Item(record_name.to_string(), name.clone())
        } else {
            panic!("wtf");
        }
    }
}

fn decompress(byte_vec: &mut ByteReader, header: &ArzRecordHeader) -> Vec<u8> {
    byte_vec.index = header.offset as usize + 24;
    //let compressed_data = &*byte_vec.read_n_bytes(header.size_compressed);
    let end = byte_vec.index + header.size_compressed as usize;
    lz4::block::decompress(&byte_vec.bytes[byte_vec.index..end], Some(header.size_decompressed.try_into().unwrap())).unwrap()
}

fn read_record_headers(byte_vec: &mut ByteReader, header: &ArzArchiveHeader) -> Vec<ArzRecordHeader> {
    let mut records = Vec::new();
    byte_vec.index = header.records_start as usize;
    for _ in 0..header.records_count {
        let record = ArzRecordHeader::read(byte_vec);
        //println!("{:?}", record);
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
