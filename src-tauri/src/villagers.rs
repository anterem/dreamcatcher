use std::collections::HashMap;

use serde::Deserialize;

use super::{AppError, LoadedSave};

// hardcoded for in-game order and to skip non villagers
static VILLAGER_ROSTER: &[u32] = &[
    // --- Valley ---
    10000130, // Aladdin
    10000131, // Jasmine
    10000132, // Alice
    10000133, // Cheshire Cat
    10000002, // Belle
    10000025, // Cogsworth
    10000007, // The Beast
    10000020, // Lumiere
    10000040, // Cinderella
    10000063, // The Fairy Godmother
    10000098, // The Forgotten
    10000099, // Mirabel
    10000034, // Anna
    10000044, // Elsa
    10000070, // Kristoff
    10000029, // Olaf
    10000187, // Hercules
    10000186, // Phil
    10000156, // Joy
    10000157, // Sadness
    10000179, // Lady
    10000180, // Tramp
    10000073, // Stitch
    10000113, // Daisy
    10000011, // Donald Duck
    10000004, // Goofy
    10000003, // Mickey Mouse
    10000046, // Minnie Mouse
    10000055, // Scrooge McDuck
    10000019, // Maui
    10000016, // Moana
    10000110, // Mike Wazowski
    10000109, // Sulley
    10000115, // Mulan
    10000116, // Mushu
    10000137, // Peter Pan
    10000182, // Pocahontas
    10000018, // Remy
    10000062, // Mother Gothel
    10000033, // Nala
    10000009, // Pumbaa
    10000061, // Scar
    10000030, // Simba
    10000010, // Timon
    10000053, // Ariel
    10000052, // Prince Eric
    10000060, // Ursula
    10000125, // Tiana
    10000035, // Merlin
    10000105, // Jack Skellington
    10000129, // Sally
    10000013, // Buzz Lightyear
    10000032, // Woody
    10000014, // WALL·E
    10000031, // Vanellope
    // --- Eternity Isle ---
    10000103, // Jafar
    10000043, // Gaston
    10000114, // Oswald
    10000027, // Rapunzel
    10000038, // EVE
    // --- Storybook Vale ---
    10000117, // Merida
    10000119, // Hades
    10000135, // Aurora
    10000127, // Maleficent
    10000059, // Flynn
    // --- Wishblossom Ranch ---
    10000162, // Cruella
    10000160, // Tinker Bell
    10000158, // Snow White
    10000166, // Tigger
];

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

    let locked = SavedCharacter::default();
    // villagers not yet in the village are dropped so unreleased arrivals are not spoiled
    let villagers = VILLAGER_ROSTER
        .iter()
        .filter_map(|&id| {
            let saved = by_id.get(&id).unwrap_or(&locked);
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
