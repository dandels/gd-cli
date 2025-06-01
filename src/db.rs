#[allow(non_camel_case_types)]
enum EntryType {
    Group(DbGroup),
    fileNameHistoryEntry(String),
}

#[allow(non_camel_case_types)]
enum GroupType {
    list,
    system,
}

struct Group {
    name: String,
    r#type: GroupType,
}
