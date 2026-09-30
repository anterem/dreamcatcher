use std::collections::{HashMap, HashSet};

use serde_json::Value;

use super::areas::biome_of;
use super::{AppError, LoadedSave};

// ActivityItem LootPresentForagingOnline is the daily moonstone chest
const MOONSTONE_CHEST_ITEM_ID: u64 = 31_400_039;

// ActivityItem TimeBending!NormalRift is an open time rift
const RIFT_ITEM_ID: u64 = 31_300_076;

// 5 moonstones per vote, 50 maximum
const DREAMSNAP_VOTE_REWARD_CAP: u64 = 10;

#[derive(Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistFacts {
    moonstone_chest_biomes: Vec<Option<String>>,
    rift_biomes: Vec<Option<String>>,
    dream_snaps: Option<DreamSnaps>,
    scrooge_stores: Vec<ScroogeStore>,
}

#[derive(Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ScroogeStore {
    location: Option<String>,
    new_items: Vec<ScroogeItem>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ScroogeItem {
    id: u32,
    name: String,
}

#[derive(Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DreamSnaps {
    submit_needed: bool,
    vote_needed: bool,
}

pub(crate) fn collect(loaded: &LoadedSave) -> Result<ChecklistFacts, AppError> {
    let save = &loaded.contents;
    Ok(ChecklistFacts {
        moonstone_chest_biomes: placed_object_biomes(save, is_moonstone_chest),
        rift_biomes: placed_object_biomes(save, is_rift),
        dream_snaps: dream_snaps(save),
        scrooge_stores: scrooge_stores(loaded),
    })
}

fn building_grid(save: &Value, building_id: u64) -> Option<(u64, String)> {
    save.pointer("/World/GridCollection/Grids")
        .and_then(|v| v.as_object())
        .and_then(|grids| {
            grids.iter().find_map(|(key, grid)| {
                let grid_id = key.parse::<u64>().ok()?;
                let has_building =
                    grid.get("Objects")
                        .and_then(|v| v.as_object())
                        .is_some_and(|objects| {
                            objects.values().any(|obj| {
                                obj.get("ItemID").and_then(|v| v.as_u64()) == Some(building_id)
                            })
                        });
                has_building.then(|| {
                    let path = grid
                        .get("GridDataPath")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    (grid_id, path)
                })
            })
        })
}

fn grid_village(save: &Value, grid_id: u64) -> Option<u64> {
    save.pointer("/World/Villages")
        .and_then(|v| v.as_array())
        .and_then(|villages| {
            villages.iter().find(|village| {
                village
                    .get("Areas")
                    .and_then(|a| a.as_object())
                    .is_some_and(|areas| {
                        areas.values().any(|area| {
                            area.get("GridIDs")
                                .and_then(|g| g.as_array())
                                .is_some_and(|ids| {
                                    ids.iter().any(|id| id.as_u64() == Some(grid_id))
                                })
                        })
                    })
            })
        })
        .and_then(|village| village.get("SceneItemId"))
        .and_then(|v| v.as_u64())
}

fn building_zone(loaded: &LoadedSave, building_id: u64) -> Option<String> {
    let save = &loaded.contents;
    let (grid_id, grid_path) = building_grid(save, building_id)?;

    let map_name = grid_village(save, grid_id).and_then(|scene_id| {
        super::game_data::cached_menu_labels(&loaded.storefront)
            .ok()
            .and_then(|labels| labels.get(&format!("label_village_{scene_id}")).cloned())
    });

    map_name.or_else(|| biome_of(&grid_path).map(str::to_string))
}

fn scrooge_stores(loaded: &LoadedSave) -> Vec<ScroogeStore> {
    let save = &loaded.contents;
    let Some(stores) = save.pointer("/World/Stores").and_then(|v| v.as_array()) else {
        return Vec::new();
    };

    let owned = owned_item_ids(save);
    let names = super::game_data::cached_item_names(&loaded.storefront).ok();
    let reset_secs = super::resets::latest_reset_secs(
        chrono::Utc::now().timestamp(),
        super::read_tz_offset(save),
        &super::resets::SCROOGE_STORE_RESET,
    );
    let mut result = Vec::new();

    for store in stores {
        let refresh_secs = super::resets::timestamp_from_field(store, "LastRefresh");
        if !super::resets::is_after_reset(refresh_secs, reset_secs) {
            continue;
        }

        let new_items = new_items_in_store(store, &owned, names.as_deref());
        if new_items.is_empty() {
            continue;
        }

        let location = store
            .get("BuildingItemID")
            .and_then(|v| v.as_u64())
            .and_then(|id| building_zone(loaded, id));
        result.push(ScroogeStore {
            location,
            new_items,
        });
    }
    result
}

fn new_items_in_store(
    store: &Value,
    owned: &HashSet<u64>,
    names: Option<&HashMap<u32, String>>,
) -> Vec<ScroogeItem> {
    let Some(displays) = store.get("Displays").and_then(|v| v.as_array()) else {
        return Vec::new();
    };

    displays
        .iter()
        .filter_map(|display| {
            display
                .pointer("/DisplayInfo/Slots")
                .and_then(|v| v.as_array())
        })
        .flatten()
        .filter_map(|slot| {
            let id = slot.pointer("/Item/id")?.as_u64()?;
            if slot.get("IsAvailable").and_then(|v| v.as_bool()) != Some(true)
                || owned.contains(&id)
            {
                return None;
            }
            let id = id as u32;
            Some(ScroogeItem {
                id,
                name: names
                    .and_then(|names| names.get(&id))
                    .cloned()
                    .unwrap_or_default(),
            })
        })
        .collect()
}
fn owned_item_ids(save: &Value) -> HashSet<u64> {
    let mut owned = HashSet::new();

    for inventory in object_values(save.pointer("/Player/ListInventories")) {
        if let Some(items) = inventory.get("Inventory").and_then(|v| v.as_object()) {
            owned.extend(items.keys().filter_map(|key| key.parse::<u64>().ok()));
        }
    }

    owned
}

fn object_values(value: Option<&Value>) -> impl Iterator<Item = &Value> {
    value
        .and_then(|v| v.as_object())
        .into_iter()
        .flat_map(|items| items.values())
}

fn placed_object_biomes(save: &Value, matches: fn(&Value) -> bool) -> Vec<Option<String>> {
    let Some(grids) = save
        .pointer("/World/GridCollection/Grids")
        .and_then(|v| v.as_object())
    else {
        return Vec::new();
    };

    let mut biomes = Vec::new();
    for grid in grids.values() {
        let Some(objects) = grid.get("Objects").and_then(|v| v.as_object()) else {
            continue;
        };

        let path = grid
            .get("GridDataPath")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let biome = biome_of(path);

        for object in objects.values() {
            if matches(object) {
                biomes.push(biome.map(|name| name.to_string()));
            }
        }
    }
    biomes
}

fn is_moonstone_chest(object: &Value) -> bool {
    object.get("ItemID").and_then(|v| v.as_u64()) == Some(MOONSTONE_CHEST_ITEM_ID)
        && object.get("State").is_none_or(|s| s.is_null())
}

fn is_rift(object: &Value) -> bool {
    object.get("ItemID").and_then(|v| v.as_u64()) == Some(RIFT_ITEM_ID)
}

fn dream_snaps(save: &Value) -> Option<DreamSnaps> {
    let stats = save
        .pointer("/Player/DesignChallenge/Stats")
        .and_then(|v| v.as_object())?;
    if stats.is_empty() {
        return None;
    }
    let current_challenge_number =
        super::resets::dreamsnap_current_challenge(chrono::Utc::now().timestamp());
    let previous_challenge_number = current_challenge_number.saturating_sub(1);
    Some(DreamSnaps {
        submit_needed: dreamsnap_entry(stats, current_challenge_number)
            .is_none_or(|c| count_field(c, "SubmitCount") < 1),
        vote_needed: dreamsnap_entry(stats, previous_challenge_number)
            .is_none_or(|c| count_field(c, "VoteCount") < DREAMSNAP_VOTE_REWARD_CAP),
    })
}

fn dreamsnap_entry(stats: &serde_json::Map<String, Value>, number: u32) -> Option<&Value> {
    stats
        .iter()
        .find_map(|(key, entry)| (challenge_number(key) == Some(number)).then_some(entry))
}

fn count_field(entry: &Value, field: &str) -> u64 {
    entry.get(field).and_then(|v| v.as_u64()).unwrap_or(0)
}

fn challenge_number(key: &str) -> Option<u32> {
    key.strip_prefix("DreamSnap")?
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::{HashMap, HashSet};

    fn store_with_slots(slots: Value) -> Value {
        json!({ "Displays": [{ "DisplayInfo": { "Slots": slots } }] })
    }

    fn names(pairs: &[(u32, &str)]) -> HashMap<u32, String> {
        pairs
            .iter()
            .map(|&(id, name)| (id, name.to_string()))
            .collect()
    }

    #[test]
    fn available_unowned_slots_report_names() {
        let store = store_with_slots(json!([
            { "Item": { "id": 40002844 }, "IsAvailable": true },
            { "Item": { "id": 50000001 }, "IsAvailable": true },
            { "Item": { "id": 50000002 }, "IsAvailable": false },
            { "Item": null, "IsAvailable": true },
        ]));
        let owned = HashSet::from([50000001]);
        let names = names(&[(40002844, "Sharpening Stone"), (50000002, "Sold Out")]);

        let items = new_items_in_store(&store, &owned, Some(&names));

        assert_eq!(
            items,
            vec![ScroogeItem {
                id: 40002844,
                name: "Sharpening Stone".to_string(),
            }]
        );
    }

    #[test]
    fn held_items_are_owned() {
        let owned = owned_item_ids(&json!({
            "Player": { "ListInventories": {
                "0": { "Inventory": { "40002859": { "Amount": 0 } } },
                "1": { "Inventory": { "50000001": { "Amount": 1 } } },
            }}
        }));

        assert!(owned.contains(&40002859));
        assert!(owned.contains(&50000001));
        assert!(!owned.contains(&40006011));
    }

    #[test]
    fn held_items_are_never_reported_new() {
        let owned = owned_item_ids(&json!({
            "Player": { "ListInventories": {
                "0": { "Inventory": { "40002859": { "Amount": 0 } } },
            }}
        }));
        let store = store_with_slots(json!([
            { "Item": { "id": 40002859 }, "IsAvailable": true },
            { "Item": { "id": 40006011 }, "IsAvailable": true },
        ]));
        let names = names(&[
            (40002859, "Weapons Rack"),
            (40006011, "Pooh's Standing Mirror"),
        ]);

        let items = new_items_in_store(&store, &owned, Some(&names));

        assert_eq!(
            items,
            vec![ScroogeItem {
                id: 40006011,
                name: "Pooh's Standing Mirror".to_string(),
            }]
        );
    }
}
