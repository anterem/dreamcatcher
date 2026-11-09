use std::collections::{BTreeSet, HashMap};

use serde::Deserialize;
use serde_json::Value;

use super::{AppError, LoadedSave};

#[derive(Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Gardening,
    Fishing,
    Mining,
    Foraging,
    Digging,
    Timebending,
    SnippetCatching,
}

fn role_from_profession_id(id: u64) -> Option<Role> {
    match id {
        1_500_000_001 => Some(Role::Gardening),
        1_500_000_002 => Some(Role::Fishing),
        1_500_000_003 => Some(Role::Mining),
        1_500_000_004 => Some(Role::Foraging),
        1_500_000_005 => Some(Role::Digging),
        1_500_000_006 => Some(Role::Timebending),
        1_500_000_007 => Some(Role::SnippetCatching),
        _ => None,
    }
}

#[derive(Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum GiftCategory {
    Produce,
    Meal,
    Flower,
    Fish,
    AnimalProduct,
    Gem,
    Material,
    Other,
}

// gift category is encoded in the item id band
fn gift_category(item_id: u32) -> GiftCategory {
    match item_id / 100_000 {
        302 => GiftCategory::Produce,
        303 => GiftCategory::Meal,
        308 => GiftCategory::Flower,
        310 => GiftCategory::Fish,
        311 => GiftCategory::AnimalProduct,
        316 => GiftCategory::Gem,
        317 => GiftCategory::Material,
        _ => GiftCategory::Other,
    }
}

#[derive(Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PreferredGift {
    pub item_id: u32,
    pub name: String,
    pub category: GiftCategory,
    pub discovered: bool,
    pub gifted: bool,
}

#[derive(Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Villager {
    pub id: u32,
    pub name: String,
    pub role: Option<Role>,
    pub friendship_level: u8,
    pub friendship_xp: u32,
    pub is_maxed: bool,
    pub gifts: Vec<PreferredGift>,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
struct SavedCharacter {
    base: SavedBase,
    status: String,
    friendship_level: u8,
    #[serde(rename = "Friendship")]
    friendship_xp: u32,
    #[serde(rename = "ProfessionID")]
    profession_id: u64,
    preferred_item_slots: Vec<SavedGiftSlot>,
    preferred_item_status: HashMap<String, String>,
}

#[derive(Default, Deserialize)]
struct SavedBase {
    id: u32,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
struct SavedGiftSlot {
    preferred_item_id: u32,
    gifted: bool,
}

impl SavedCharacter {
    fn in_village(&self) -> bool {
        self.status == "CharacterStatus_InVillage"
    }

    fn gifts(&self, names: &HashMap<u32, String>) -> Vec<PreferredGift> {
        self.preferred_item_slots
            .iter()
            .map(|slot| {
                let item_id = slot.preferred_item_id;
                let discovered = self
                    .preferred_item_status
                    .get(&item_id.to_string())
                    .is_some_and(|s| s == "PreferredItemStatus_Discovered");
                PreferredGift {
                    item_id,
                    name: names.get(&item_id).cloned().unwrap_or_default(),
                    category: gift_category(item_id),
                    discovered,
                    gifted: slot.gifted,
                }
            })
            .collect()
    }
}

// Player/CollectionSets contains all villager ids
fn collection_villager_ids(save: &Value) -> BTreeSet<u32> {
    let Some(sets) = save
        .pointer("/Player/CollectionSets")
        .and_then(Value::as_array)
    else {
        return BTreeSet::new();
    };
    sets.iter()
        .filter_map(|set| set.get("GroupData")?.as_array())
        .flatten()
        .filter_map(|group| group.get("GroupsCollectionItems")?.as_object())
        .flat_map(|items| items.keys())
        .filter_map(|id| id.parse::<u32>().ok())
        .filter(|id| (10_000_000..10_100_000).contains(id))
        .collect()
}

pub(crate) fn collect(loaded: &LoadedSave) -> Result<Vec<Villager>, AppError> {
    let names = super::game_data::cached_item_names(&loaded.storefront)?;

    let by_id: HashMap<u32, SavedCharacter> = loaded
        .contents
        .pointer("/World/Characters")
        .and_then(|v| v.as_array())
        .map(|chars| {
            chars
                .iter()
                .filter_map(|c| SavedCharacter::deserialize(c).ok())
                .map(|c| (c.base.id, c))
                .collect()
        })
        .unwrap_or_default();

    let mut villager_ids = collection_villager_ids(&loaded.contents);
    if villager_ids.is_empty() {
        // saves without character collections: trust the live status instead
        villager_ids.extend(by_id.values().filter(|c| c.in_village()).map(|c| c.base.id));
    }

    // villagers not yet in the village are dropped so unreleased arrivals are not spoiled
    let villagers = villager_ids
        .into_iter()
        .filter_map(|id| {
            let saved = by_id.get(&id)?;
            if !saved.in_village() {
                return None;
            }
            Some(Villager {
                id,
                name: names.get(&id).cloned().unwrap_or_default(),
                role: role_from_profession_id(saved.profession_id),
                friendship_level: saved.friendship_level,
                friendship_xp: saved.friendship_xp,
                is_maxed: saved.friendship_level == 10,
                gifts: saved.gifts(&names),
            })
        })
        .collect();

    Ok(villagers)
}
