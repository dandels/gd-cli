use std::path::PathBuf;
use std::fs::File;
use std::io::Error;
use std::io::Read;
use super::ByteVec;

const HEADER_SIZE: u8 = 28;

#[derive(Debug, Clone)]
pub struct ArcRecordHeader {
    pub record_type: u32,
    pub offset: u32,
    pub len_compresssed: u32,
    pub len_decompresssed: u32,
    pub unknown: u32,
    pub filetime: u64,
    pub parts_count: u32,
    pub index: u32,
    pub str_len: u32,
    pub str_offset: u32,
}

#[derive(Debug)]
pub struct ArcRecord{
    pub header: ArcRecordHeader,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ArcHeader {
    #[allow(dead_code)]
    pub unknown: u32,
    pub version: u32,
    pub files_count: u32,
    pub records_count: u32,
    pub record_len: u32,
    pub string_table_len: u32,
    pub record_offset: u32,
}

impl ArcHeader {
    fn new(byte_vec: &mut ByteVec) -> Self {
        Self {
            unknown: byte_vec.read_u32(),
            version: byte_vec.read_u32(),
            files_count: byte_vec.read_u32(),
            records_count: byte_vec.read_u32(),
            record_len: byte_vec.read_u32(),
            string_table_len: byte_vec.read_u32(),
            record_offset: byte_vec.read_u32(),
        }
    }
}

#[derive(Debug)]
struct ArcFilePart {
    pub offset: u32,
    pub len_compressed: u32,
    pub len_decompressed: u32,
}

impl ArcFilePart {
    pub fn new(byte_vec: &mut ByteVec) -> Self {
        Self {
            offset: byte_vec.read_u32(),
            len_compressed: byte_vec.read_u32(),
            len_decompressed: byte_vec.read_u32(),
        }
    }
}

pub struct ArcParser {
    pub data: Vec<Vec<u8>>,
}

impl ArcParser {
    pub fn parse_templates(path: &PathBuf) -> Result<Vec<ArcRecord>, Error> {
        let mut byte_vec = ByteVec::new(path)?;
        let header = ArcHeader::new(&mut byte_vec);
        assert!(header.version == 3, "expected header version 3, is {}", header.version);
        let mut file_parts: Vec<ArcFilePart> = Vec::with_capacity(header.records_count as usize);
        byte_vec.index = header.record_offset as usize;

        for _ in 0..header.records_count {
            file_parts.push(ArcFilePart::new(&mut byte_vec));
        }

        for part in &file_parts {
            println!("part {:?}", part);
        }

        let _strings = Self::read_strings(&mut byte_vec, &header);
        //for string in strings {
        //    println!("{}", string);
        //}
        let record_headers = Self::read_record_headers(&mut byte_vec, &header);
        //for record in records {
        //    println!("{:?}", record);
        //}
        println!("records done");
        let mut data: Vec<ArcRecord> = Vec::new();
        assert_eq!(header.files_count as usize, record_headers.len());
        for i in 0..header.files_count {
            data.push(ArcRecord {
                header: record_headers[i as usize].clone(),
                data: Self::decompress(&mut byte_vec, &file_parts),
            });
        }
        Ok(
            data
        )
    }

    fn read_strings(byte_vec: &mut ByteVec, header: &ArcHeader) -> Vec<String> {
        let mut strings = Vec::new();
        byte_vec.index = (header.record_offset + header.record_len) as usize;
        //byte_vec.read_string(header.string_table_len);

        for i in 0..header.files_count {
            let string = byte_vec.read_cstring();
            //println!("{string}");
            strings.push(string);
        }
        println!("strings done");
        strings
    }

    fn read_record_headers(byte_vec: &mut ByteVec, header: &ArcHeader) -> Vec<ArcRecordHeader> {
        let mut records = Vec::new();
        byte_vec.index = (header.record_offset + header.record_len + header.string_table_len) as usize;
        for _ in 0..header.files_count {
            records.push(ArcRecordHeader {
                record_type: byte_vec.read_u32(),
                offset: byte_vec.read_u32(),
                len_compresssed: byte_vec.read_u32(),
                len_decompresssed: byte_vec.read_u32(),
                unknown: byte_vec.read_u32(),
                filetime: byte_vec.read_u64(),
                parts_count: byte_vec.read_u32(),
                index: byte_vec.read_u32(),
                str_len: byte_vec.read_u32(),
                str_offset: byte_vec.read_u32(),
            });
        }
        records
    }

    fn decompress(byte_vec: &mut ByteVec, parts: &Vec<ArcFilePart>) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        for part in parts {
            byte_vec.index = part.offset as usize;
            if part.len_compressed == part.len_decompressed {
                data.append(&mut byte_vec.read_n_bytes(part.len_compressed).to_vec());
            } else {
                let mut buf = vec![0; part.len_decompressed as usize];
                let compressed_data = &*byte_vec.read_n_bytes(part.len_compressed);
                lz4::block::decompress_to_buffer(compressed_data, Some(part.len_decompressed.try_into().unwrap()), &mut buf).unwrap();
                data.append(&mut buf.to_vec());
            }
        }
        data
    }
}
