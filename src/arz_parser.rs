use crate::byte_vec::ByteVec;
use std::path::PathBuf;
use std::io::Error;

struct Record {
    string_index: u32,
    record_type: String,
    offset: u32,
    size_decompressed: u32,
}

impl Record {
    fn read(byte_vec: &mut ByteVec) -> Self {
        Self {
            string_index: byte_vec.read_u32(),
            record_type: byte_vec.read_cstring(),
            offset: byte_vec.read_u32(),
            size_decompressed: byte_vec.read_u32(),
        }
    }
}

// v3 of the header?
struct ArzHeader {
    unknown: u16, // Item Assistant code thinks this is the version check?
    version: u16,
    records_start: u32,
    records_len: u32,
    records_count: u32,
    strings_start: u32,
    strings_size: u32,
}

impl ArzHeader {
    fn new(byte_vec: &mut ByteVec) -> Self {
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

pub struct ArzParser {
}

impl ArzParser {
    pub fn parse_database(path: &PathBuf) -> Result<(), Error> {
        let mut byte_vec = ByteVec::new(path)?;
        let header = ArzHeader::new(&mut byte_vec);

        // Asserts copied from IA example
        assert_eq!(header.unknown, 2);
        assert_eq!(header.version, 3);

        for i in 0..header.records_count {
        }
        Ok(())
    }

    fn load_records(byte_vec: &mut ByteVec, header: &ArzHeader) -> Vec<Record> {
        let mut records = Vec::new();
        byte_vec.index = header.records_start as usize;

        for _ in 0..header.records_len {
            records.push(Record::read(byte_vec));
        }

        records
    }
}
