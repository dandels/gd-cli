use crate::inventory_item::InventoryItem;
use std::sync::RwLock;
use std::sync::Arc;
use std::{fmt, fmt::Display};
use crate::arc_parser::ArcParser;
use crate::arz_parser::*;

pub struct ItemLookup {
    pub search_term: String,
    pub localization_data: Arc<RwLock<ArcParser>>,
    pub tag_names: Arc<RwLock<ArzParser>>,
}

pub struct CompleteItem {
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
    pub fn lookup_item(&self, inventory_item: &InventoryItem) -> Option<CompleteItem> {
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

    pub fn check_item(&self, inventory_item: &InventoryItem, item_source: &str) {
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

