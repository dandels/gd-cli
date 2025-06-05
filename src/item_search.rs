use crate::arz_parser::EntryType;
use crate::inventory_item::InventoryItem;

use std::collections::HashMap;
use std::{fmt, fmt::Display};

use colored::{ColoredString, Colorize};

pub type LocalizationStrings = HashMap<String, String>;

#[derive(Debug, Default)]
pub struct TagNames {
    pub items: HashMap<String, EntryType>,
    pub affixes: HashMap<String, EntryType>,
}

pub struct ItemLookup {
    pub search_term: String,
    pub localization_data: HashMap<String, String>,
    pub tag_names: TagNames,
}

pub struct CompleteItem {
    name: ColoredString,
    prefix: Option<ColoredString>,
    suffix: Option<ColoredString>,
    level_req: Option<u32>,
    quantity: u32,
}

impl CompleteItem {}

impl Display for CompleteItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let quantity = {
            if self.quantity > 1 {
                format!("(x{}) ", self.quantity)
            } else {
                "".to_string()
            }
        };
        let lvl_req = {
            if let Some(req) = self.level_req {
                format!("[lvl {req}]")
            } else {
                "".to_string()
            }
        };

        let name = &self.name;
        if self.prefix.is_none() {
            write!(
                f,
                "{lvl_req} {quantity}{name} {}",
                self.suffix.as_ref().unwrap_or(&"".into())
            )
        } else {
            write!(
                f,
                "{lvl_req} {} {name} {}",
                self.prefix.as_ref().unwrap(),
                self.suffix.as_ref().unwrap_or(&"".into())
            )
        }
    }
}

impl ItemLookup {
    pub fn lookup_item(&self, inventory_item: &InventoryItem) -> Option<CompleteItem> {
        if let Some(EntryType::Item(_record_name, tag_name, rarity, level_req)) =
            self.tag_names.items.get(&inventory_item.base_name)
        {
            if let Some(item_name) = self.localization_data.get(tag_name) {
                // Clone instead of get_mut() since localization_data is shared between threads
                let mut item_name = item_name.clone();
                let colored_name = match rarity.as_str() {
                    "Legendary" => item_name.purple(),
                    "Rare" => {
                        // Rare components have this for some reason...
                        if item_name.starts_with("^k") {
                            item_name.drain(0..2);
                            item_name.yellow()
                        } else {
                            item_name.bright_green()
                        }
                    }
                    "Epic" => item_name.bright_blue(),
                    "Magical" => item_name.bright_yellow(),
                    // ... is there really no API to construct a ColoredString without a style..?
                    _ => {
                        let mut cs = item_name.red();
                        cs.fgcolor = None;
                        cs
                    }
                };
                let mut prefix: Option<String> = None;
                let mut colored_prefix: Option<ColoredString> = None;
                if !inventory_item.prefix_name.is_empty() {
                    let tag_prefix = self.tag_names.affixes.get(&inventory_item.prefix_name);
                    if let Some(EntryType::Affix(affix_info)) = tag_prefix {
                        if let Some(affix_name) = &affix_info.name {
                            prefix = Some(affix_name.clone());
                        } else if let Some(tag_name) = &affix_info.tag_name {
                            if let Some(name) = self.localization_data.get(tag_name) {
                                prefix = Some(name.clone());
                            }
                        }
                        if let Some(p) = prefix {
                            if affix_info.rarity.to_lowercase() == "rare" {
                                colored_prefix = Some(p.green());
                            } else if affix_info.rarity.to_lowercase() == "magical" {
                                colored_prefix = Some(p.yellow());
                            }
                        }
                    }
                }
                let mut suffix = None;
                let mut colored_suffix = None;
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
                        if let Some(s) = suffix {
                            if affix_info.rarity.to_lowercase() == "rare" {
                                colored_suffix = Some(s.green());
                            } else if affix_info.rarity.to_lowercase() == "magical" {
                                colored_suffix = Some(s.yellow());
                            } else {
                                println!("{}", affix_info.rarity);
                            }
                        }
                    }
                }
                let quantity = inventory_item.stack_count;
                Some(CompleteItem {
                    name: colored_name,
                    prefix: colored_prefix,
                    suffix: colored_suffix,
                    level_req: *level_req,
                    quantity,
                })
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
                // Most of print logic is handled inside CompleteItem
                println!("{item_source}: {ci}");
            }
        // There are some items with blank fields that might be unused assets. Otherwise log an error.
        } else if !inventory_item.base_name.is_empty() {
            println!("No tag found for {}", inventory_item.base_name);
        }
    }
}
