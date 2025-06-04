use crate::inventory_item::InventoryItem;
use crate::arz_parser::EntryType;

use std::{fmt, fmt::Display};
use std::collections::HashMap;

pub type LocalizationStrings = HashMap<String, String>;

#[derive(Debug, Default)]
pub struct TagNames {
    pub items: HashMap<String, EntryType>,
    pub affixes: HashMap<String, EntryType>
}

pub struct ItemLookup {
    pub search_term: String,
    pub localization_data: HashMap<String, String>,
    pub tag_names: TagNames,
}

pub struct CompleteItem {
    name: String,
    prefix: Option<String>,
    suffix: Option<String>,
    quantity: u32,
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
        if let Some(EntryType::Item(_record_name, tag_name)) = self.tag_names.items.get(&inventory_item.base_name) {
            if let Some(name) = self.localization_data.get(tag_name) {
                let mut prefix = None;
                if !inventory_item.prefix_name.is_empty() {
                    let tag_prefix = self.tag_names.affixes.get(&inventory_item.prefix_name);
                    if let Some(EntryType::Affix(affix_info)) = tag_prefix {
                        if let Some(name) = &affix_info.name {
                            prefix = Some(name.clone());
                        } else if let Some(tag_name) = &affix_info.tag_name {
                            if let Some(name) = self.localization_data.get(tag_name) {
                                prefix = Some(name.clone());
                            }
                        }
                    }
                }
                let mut suffix = None;
                if !inventory_item.suffix_name.is_empty() {
                    let tag_suffix = self.tag_names.affixes.get(&inventory_item.suffix_name);
                    if let Some(EntryType::Affix(affix_info)) = tag_suffix {
                        if let Some(name) = &affix_info.name {
                            suffix = Some(name.clone());
                        } else if let Some(tag_name) = &affix_info.tag_name {
                            if let Some(name) = self.localization_data.get(tag_name) {
                                suffix = Some(name.clone());
                            }
                        }
                    }
                }
                let quantity = inventory_item.stack_count;
                Some(CompleteItem { name: name.clone(), prefix, suffix, quantity })
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
                if ci.quantity > 1 {
                    println!("{item_source}: {}x {item_name}", ci.quantity);
                } else {
                    println!("{item_source}: {item_name}");
                }
            }
        // There are some items with blank fields that might be unused assets. Otherwise log an error.
        } else if !inventory_item.base_name.is_empty() {
            println!("No tag found for {}", inventory_item.base_name);
        }
    }
}

