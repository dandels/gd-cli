mod arc_parser;
mod arz_parser;
mod byte_vec;
mod item;
mod stash;

use arc_parser::ArcParser;
use byte_vec::ByteVec;

use std::io::Error;
use std::path::PathBuf;

fn main() -> Result<(), Error> {
    //let stash_path = std::path::PathBuf::from("transfer.gst");
    //let _stash = Stash::new(&stash_path)?;

    //for tab in stash.tabs {
    //    for item in tab {
    //        println!("{:?}", item)
    //    }
    //}

    let path = PathBuf::from("/home/dee/games/Grim Dawn/database/templates.arc");
    let templates = ArcParser::parse_templates(&path).unwrap();
    for record in templates {
        let string = String::from_utf8(record.data).unwrap();
        println!("{string}");
    }

    Ok(())
}
