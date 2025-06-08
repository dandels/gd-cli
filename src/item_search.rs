use crate::arz_parser::EntryType;
use crate::inventory_item::InventoryItem;

use std::collections::HashMap;
use std::{fmt, fmt::Display};

use colored::{ColoredString, Colorize};

pub type LocalizationStrings = HashMap<String, String>;

#[derive(Debug, Default)]
pub struct TagNames {
    pub items: HashMap<String, (EntryType, Vec<Option<u32>>)>,
    pub affixes: HashMap<String, EntryType>,
}

pub struct ItemLookup {
    pub search_term: String,
    pub localization_data: HashMap<String, String>,
    pub tag_names: TagNames,
}

pub struct CompleteItem {
    name: String,
    item_rarity: Rarity,
    prefix: Option<String>,
    prefix_rarity: Rarity,
    suffix: Option<String>,
    suffix_rarity: Rarity,
    level_req: Option<u32>,
    quantity: u32,
}

enum Rarity {
    Legendary,
    Epic,
    Rare,
    RareComponent,
    Magical,
    CommonOrUnknown,
}

impl From<&String> for Rarity {
    fn from(string: &String) -> Self {
        match string.to_lowercase().as_str() {
            "legendary" => Self::Legendary,
            "rare" => Self::Rare,
            "epic" => Self::Epic,
            "magical" => Self::Magical,
            _ => Self::CommonOrUnknown,
        }
    }
}

fn color_item_by_rarity(string: String, rarity: &Rarity) -> ColoredString {
    match rarity {
        Rarity::Legendary => string.purple(),
        Rarity::Epic => string.bright_blue(),
        Rarity::Rare => string.bright_green(),
        Rarity::RareComponent => string.yellow(),
        Rarity::Magical => string.bright_yellow(),
        Rarity::CommonOrUnknown => ColoredString::from(string),
    }
}

fn color_affix_by_rarity(string: String, rarity: &Rarity) -> ColoredString {
    match rarity {
        Rarity::Rare => string.green(),
        Rarity::Magical => string.yellow(),
        _ => ColoredString::from(string),
    }
}

impl CompleteItem {
    fn fmt_searchable_item_name(&self) -> String {
        format!(
            "{} {} {}", // correct amount of whitespace is not important for search
            self.prefix.as_ref().unwrap_or(&"".into()),
            &self.name,
            self.suffix.as_ref().unwrap_or(&"".into())
        )
    }
}

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

        let name_colored = color_item_by_rarity(self.name.clone(), &self.item_rarity);
        let suffix_colored = color_affix_by_rarity(self.suffix.clone().unwrap_or("".to_string()), &self.suffix_rarity);
        let prefix_colored = color_affix_by_rarity(self.prefix.clone().unwrap_or("".to_string()), &self.prefix_rarity);
        if self.prefix.is_none() {
            if self.suffix.is_none() {
                write!(f, "{lvl_req} {quantity}{name_colored}")
            } else {
                write!(f, "{lvl_req} {quantity}{name_colored} {suffix_colored}")
            }
        } else if self.suffix.is_none() {
            write!(f, "{lvl_req} {prefix_colored} {name_colored}")
        } else {
            write!(f, "{lvl_req} {prefix_colored} {name_colored} {suffix_colored}")
        }
    }
}

impl ItemLookup {
    pub fn lookup_item(&self, inventory_item: &InventoryItem) -> Option<CompleteItem> {
        if let Some((EntryType::Item(_record_name, tag_name, item_rarity, level_req), ilvls)) =
            self.tag_names.items.get(&inventory_item.base_name)
        {
            if let Some(item_name) = self.localization_data.get(tag_name) {
                // Uncomment to get record name and tag name of an item that the player has
                //if item_name == "Baldir's Mantle" {
                //    //println!("mantle is {record_name}, {tag_name}");
                //    println!("{:?}", inventory_item);
                //}
                // TODO fix this logic, hashmap needs to count tagnames and not record names
                if ilvls.len() > 1 {
                    println!("{item_name} has {} tiers", ilvls.len());
                }

                let mut prefix: Option<String> = None;
                let mut prefix_rarity = Rarity::CommonOrUnknown;
                if !inventory_item.prefix_name.is_empty() {
                    let tag_prefix = self.tag_names.affixes.get(&inventory_item.prefix_name);
                    if let Some(EntryType::Affix(affix_info)) = tag_prefix {
                        prefix_rarity = Rarity::from(&affix_info.rarity);
                        if let Some(affix_name) = &affix_info.name {
                            prefix = Some(affix_name.clone());
                        } else if let Some(tag_name) = &affix_info.tag_name {
                            if let Some(name) = self.localization_data.get(tag_name) {
                                prefix = Some(name.clone());
                            }
                        }
                    }
                }
                let mut suffix = None;
                let mut suffix_rarity = Rarity::CommonOrUnknown;
                if !inventory_item.suffix_name.is_empty() {
                    let tag_suffix = self.tag_names.affixes.get(&inventory_item.suffix_name);
                    if let Some(EntryType::Affix(affix_info)) = tag_suffix {
                        suffix_rarity = Rarity::from(&affix_info.rarity);
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

                let mut item_name = item_name.clone();
                let item_rarity = {
                    /* Rare components have this for some reason... Let's give them their own color since we aren't
                     * detecting them in any other way. */
                    if item_name.starts_with("^k") {
                        item_name.drain(0..2);
                        Rarity::RareComponent
                    } else {
                        Rarity::from(item_rarity)
                    }
                };

                Some(CompleteItem {
                    name: item_name,
                    item_rarity,
                    prefix,
                    prefix_rarity,
                    suffix,
                    suffix_rarity,
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
            let item_name = ci.fmt_searchable_item_name();
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
