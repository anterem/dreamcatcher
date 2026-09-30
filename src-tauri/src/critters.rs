use specta_typescript::Number;
use std::collections::{HashMap, HashSet};

const SPARK_OF_IMAGINATION_MISSION: u64 = 2_070_001_037;

// bitmask - 0 to 23 (local time)
type AvailableHours = u32;

const fn hours(start: u32, end: u32) -> AvailableHours {
    assert!(start < end && end <= 24);
    (1u32 << end) - (1 << start)
}

enum Note {
    SparkOfImagination,
    ClearWeather,
    RainyWeather,
}

impl Note {
    fn as_str(&self) -> &'static str {
        match self {
            Note::SparkOfImagination => "Requires The Spark of Imagination quest (A Rift In Time)",
            Note::ClearWeather => "Only during clear or light cloudy weather",
            Note::RainyWeather => "Only during heavy clouds or rainy weather",
        }
    }

    fn required_mission(&self) -> Option<u64> {
        match self {
            Note::SparkOfImagination => Some(SPARK_OF_IMAGINATION_MISSION),
            _ => None,
        }
    }
}

struct CritterDef {
    id: u32,
    species: &'static str,
    biome: &'static str,
    days: [AvailableHours; 7],
    notes: &'static [Note],
}

const NA: AvailableHours = 0;
const AM: AvailableHours = hours(0, 12);
const PM: AvailableHours = hours(12, 24);
const ALL: AvailableHours = hours(0, 24);

static CRITTER_DATA: &[CritterDef] = &[
    CritterDef {
        id: 120100025, // Emerald Sunbird
        species: "Sunbirds",
        biome: "Sunlit Plateau",
        days: [PM, NA, ALL, ALL, NA, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100026, // Golden Sunbird
        species: "Sunbirds",
        biome: "Sunlit Plateau",
        days: [AM, NA, ALL, NA, ALL, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100027, // Orchid Sunbird
        species: "Sunbirds",
        biome: "Sunlit Plateau",
        days: [NA, NA, NA, NA, NA, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100005, // Red Sunbird
        species: "Sunbirds",
        biome: "Sunlit Plateau",
        days: [PM, ALL, NA, NA, ALL, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100024, // Turquoise Sunbird
        species: "Sunbirds",
        biome: "Sunlit Plateau",
        days: [AM, ALL, NA, ALL, NA, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100010, // Black Squirrel
        species: "Squirrels",
        biome: "Plaza",
        days: [AM, NA, ALL, NA, ALL, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100001, // Classic Squirrel
        species: "Squirrels",
        biome: "Plaza",
        days: [PM, ALL, NA, NA, ALL, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100009, // Gray Squirrel
        species: "Squirrels",
        biome: "Plaza",
        days: [PM, NA, ALL, ALL, NA, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100008, // Red Squirrel
        species: "Squirrels",
        biome: "Plaza",
        days: [AM, ALL, NA, ALL, NA, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100011, // White Squirrel
        species: "Squirrels",
        biome: "Plaza",
        days: [ALL, NA, NA, NA, NA, NA, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100028, // Blue Crocodile
        species: "Crocodiles",
        biome: "Glade of Trust",
        days: [AM, ALL, NA, ALL, NA, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100007, // Classic Crocodile
        species: "Crocodiles",
        biome: "Glade of Trust",
        days: [PM, ALL, NA, NA, ALL, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100030, // Golden Crocodile
        species: "Crocodiles",
        biome: "Glade of Trust",
        days: [AM, NA, ALL, NA, ALL, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100040, // Pink Crocodile
        species: "Crocodiles",
        biome: "Glade of Trust",
        days: [NA, NA, NA, NA, NA, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100029, // Red Crocodile
        species: "Crocodiles",
        biome: "Glade of Trust",
        days: [PM, NA, ALL, ALL, NA, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100031, // White Crocodile
        species: "Crocodiles",
        biome: "Glade of Trust",
        days: [ALL, NA, NA, NA, NA, NA, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100012, // Black Rabbit
        species: "Rabbits",
        biome: "Peaceful Meadow",
        days: [AM, ALL, NA, ALL, NA, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100013, // Brown Rabbit
        species: "Rabbits",
        biome: "Peaceful Meadow",
        days: [PM, NA, ALL, ALL, NA, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100015, // Calico Rabbit
        species: "Rabbits",
        biome: "Peaceful Meadow",
        days: [NA, NA, NA, NA, ALL, NA, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100000, // Classic Rabbit
        species: "Rabbits",
        biome: "Peaceful Meadow",
        days: [PM, ALL, NA, NA, ALL, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100014, // White Rabbit
        species: "Rabbits",
        biome: "Peaceful Meadow",
        days: [AM, NA, ALL, NA, ALL, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100034, // Blue Raven
        species: "Ravens",
        biome: "Forgotten Lands",
        days: [AM, NA, ALL, NA, ALL, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100035, // Brown Raven
        species: "Ravens",
        biome: "Forgotten Lands",
        days: [NA, NA, ALL, NA, NA, NA, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100004, // Classic Raven
        species: "Ravens",
        biome: "Forgotten Lands",
        days: [PM, ALL, NA, NA, ALL, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100032, // Red Raven
        species: "Ravens",
        biome: "Forgotten Lands",
        days: [AM, ALL, NA, ALL, NA, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100033, // White Raven
        species: "Ravens",
        biome: "Forgotten Lands",
        days: [PM, NA, ALL, ALL, NA, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100019, // Black Sea Turtle
        species: "Sea Turtles",
        biome: "Dazzle Beach",
        days: [NA, ALL, NA, NA, NA, NA, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100016, // Brown Sea Turtle
        species: "Sea Turtles",
        biome: "Dazzle Beach",
        days: [AM, ALL, NA, ALL, NA, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100002, // Classic Sea Turtle
        species: "Sea Turtles",
        biome: "Dazzle Beach",
        days: [PM, ALL, NA, NA, ALL, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100018, // Purple Sea Turtle
        species: "Sea Turtles",
        biome: "Dazzle Beach",
        days: [AM, NA, ALL, NA, ALL, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100017, // White Sea Turtle
        species: "Sea Turtles",
        biome: "Dazzle Beach",
        days: [PM, NA, ALL, ALL, NA, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100037, // Black Fox
        species: "Foxes",
        biome: "Frosted Heights",
        days: [PM, NA, ALL, ALL, NA, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100038, // Blue Fox
        species: "Foxes",
        biome: "Frosted Heights",
        days: [AM, NA, ALL, NA, ALL, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100036, // Classic Fox
        species: "Foxes",
        biome: "Frosted Heights",
        days: [AM, ALL, NA, ALL, NA, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100039, // Red Fox
        species: "Foxes",
        biome: "Frosted Heights",
        days: [NA, NA, NA, NA, NA, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100006, // White Fox
        species: "Foxes",
        biome: "Frosted Heights",
        days: [PM, ALL, NA, NA, ALL, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100021, // Black Raccoon
        species: "Raccoons",
        biome: "Forest of Valor",
        days: [PM, NA, ALL, ALL, NA, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100023, // Blue Raccoon
        species: "Raccoons",
        biome: "Forest of Valor",
        days: [NA, NA, NA, ALL, NA, NA, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100003, // Classic Raccoon
        species: "Raccoons",
        biome: "Forest of Valor",
        days: [PM, ALL, NA, NA, ALL, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100020, // Red Raccoon
        species: "Raccoons",
        biome: "Forest of Valor",
        days: [AM, ALL, NA, ALL, NA, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100022, // White Raccoon
        species: "Raccoons",
        biome: "Forest of Valor",
        days: [AM, NA, ALL, NA, ALL, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100042, // Black and White Capybara
        species: "Capybaras",
        biome: "The Promenade",
        days: [PM, NA, ALL, NA, ALL, PM, PM],
        notes: &[],
    },
    CritterDef {
        id: 120100043, // Blue Striped Capybara
        species: "Capybaras",
        biome: "The Grove",
        days: [NA, AM, AM, AM, ALL, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100041, // Classic Capybara
        species: "Capybaras",
        biome: "The Grasslands",
        days: [ALL; 7],
        notes: &[],
    },
    CritterDef {
        id: 120100044, // Gray Spotted Capybara
        species: "Capybaras",
        biome: "The Lagoon",
        days: [ALL, PM, PM, PM, NA, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100045, // Red and White Striped Capybara
        species: "Capybaras",
        biome: "The Lagoon",
        days: [NA, NA, NA, NA, NA, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100057, // Toon Capybara
        species: "Capybaras",
        biome: "The Grove",
        days: [ALL; 7],
        notes: &[Note::SparkOfImagination],
    },
    CritterDef {
        id: 120100047, // Blue and Red Striped Cobra
        species: "Cobras",
        biome: "The Wastes",
        days: [ALL, PM, PM, NA, ALL, NA, PM],
        notes: &[],
    },
    CritterDef {
        id: 120100046, // Classic Cobra
        species: "Cobras",
        biome: "The Plains",
        days: [ALL; 7],
        notes: &[],
    },
    CritterDef {
        id: 120100050, // Green and White Striped Cobra
        species: "Cobras",
        biome: "The Borderlands",
        days: [NA, NA, NA, NA, ALL, NA, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100048, // Pink Spotted Cobra
        species: "Cobras",
        biome: "The Oasis",
        days: [AM, ALL, NA, AM, AM, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100056, // Toon Cobra
        species: "Cobras",
        biome: "The Wastes",
        days: [ALL; 7],
        notes: &[Note::SparkOfImagination],
    },
    CritterDef {
        id: 120100049, // Yellow and Purple Striped Cobra
        species: "Cobras",
        biome: "The Borderlands",
        days: [PM, NA, ALL, PM, PM, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100054, // Beige Monkey
        species: "Monkeys",
        biome: "The Ruins",
        days: [ALL, PM, NA, PM, ALL, PM, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100052, // Black and Brown Monkey
        species: "Monkeys",
        biome: "The Courtyard",
        days: [PM, NA, PM, ALL, PM, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100055, // Black and Gray Monkey
        species: "Monkeys",
        biome: "The Ruins",
        days: [NA, NA, ALL, NA, NA, NA, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100051, // Classic Monkey
        species: "Monkeys",
        biome: "The Docks",
        days: [ALL; 7],
        notes: &[],
    },
    CritterDef {
        id: 120100053, // Red and Beige Monkey
        species: "Monkeys",
        biome: "The Overlook",
        days: [NA, AM, ALL, AM, NA, AM, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100058, // Toon Monkey
        species: "Monkeys",
        biome: "The Docks",
        days: [ALL; 7],
        notes: &[Note::SparkOfImagination],
    },
    CritterDef {
        id: 120100062, // Brown Owl
        species: "Owls",
        biome: "The Library of Lore",
        days: [NA, ALL, ALL, ALL, ALL, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100072, // Dark Owl
        species: "Owls",
        biome: "The Library of Lore",
        days: [ALL, NA, NA, NA, NA, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100073, // White Owl
        species: "Owls",
        biome: "The Bind",
        days: [hours(15, 20); 7],
        notes: &[],
    },
    CritterDef {
        id: 120100063, // Purple Owl
        species: "Owls",
        biome: "The Bind",
        days: [hours(0, 9); 7],
        notes: &[],
    },
    CritterDef {
        id: 120100068, // Blue Baby Dragon
        species: "Baby Dragons",
        biome: "The Wild Woods",
        days: [NA, NA, ALL, ALL, ALL, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100070, // Green Baby Dragon
        species: "Baby Dragons",
        biome: "The Fallen Fortress",
        days: [
            hours(10, 18),
            hours(10, 18),
            hours(10, 18),
            hours(10, 18),
            hours(10, 18),
            NA,
            NA,
        ],
        notes: &[],
    },
    CritterDef {
        id: 120100071, // Purple Baby Dragon
        species: "Baby Dragons",
        biome: "The Beanstalk Marshes",
        days: [AM; 7],
        notes: &[],
    },
    CritterDef {
        id: 120100069, // Red Baby Dragon
        species: "Baby Dragons",
        biome: "Teapot Falls",
        days: [ALL, NA, NA, NA, NA, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100083, // Teal Baby Dragon
        species: "Baby Dragons",
        biome: "The Wild Woods",
        days: [ALL, NA, NA, ALL, ALL, ALL, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100084, // Black Pegasus
        species: "Pegasi",
        biome: "The Elysian Fields",
        days: [ALL, NA, NA, ALL, ALL, ALL, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100065, // Blue Pegasus
        species: "Pegasi",
        biome: "The Fiery Plains",
        days: [NA, ALL, ALL, ALL, NA, NA, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100067, // Peach Pegasus
        species: "Pegasi",
        biome: "Mount Olympus",
        days: [hours(6, 14); 7],
        notes: &[],
    },
    CritterDef {
        id: 120100064, // Pink Pegasus
        species: "Pegasi",
        biome: "The Elysian Fields",
        days: [ALL, NA, NA, NA, ALL, ALL, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100066, // Yellow Pegasus
        species: "Pegasi",
        biome: "The Statue's Shadow",
        days: [
            NA,
            NA,
            hours(18, 24),
            hours(18, 24),
            hours(18, 24),
            hours(18, 24),
            hours(18, 24),
        ],
        notes: &[],
    },
    CritterDef {
        id: 120100080, // Black Goose
        species: "Geese",
        biome: "Silver Summit",
        days: [hours(5, 11); 7],
        notes: &[],
    },
    CritterDef {
        id: 120100096, // Blue Goose
        species: "Geese",
        biome: "Wishing Way",
        days: [hours(18, 20); 7],
        notes: &[],
    },
    CritterDef {
        id: 120100081, // Golden Goose
        species: "Geese",
        biome: "Delver Dale",
        days: [hours(7, 8) | hours(19, 20); 7],
        notes: &[],
    },
    CritterDef {
        id: 120100078, // Goose
        species: "Geese",
        biome: "Wishblossom Ranch",
        days: [ALL, ALL, NA, ALL, NA, ALL, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100079, // True North Goose
        species: "Geese",
        biome: "Ranch Highlands",
        days: [NA, NA, ALL, NA, ALL, NA, ALL],
        notes: &[],
    },
    CritterDef {
        id: 120100086, // Brown Skunk
        species: "Skunks",
        biome: "Paisley Park",
        days: [PM, PM, NA, PM, NA, PM, NA],
        notes: &[],
    },
    CritterDef {
        id: 120100088, // Patterned Skunk
        species: "Skunks",
        biome: "Haute Plateau",
        days: [ALL; 7],
        notes: &[Note::RainyWeather],
    },
    CritterDef {
        id: 120100085, // Skunk
        species: "Skunks",
        biome: "Runway River",
        days: [AM; 7],
        notes: &[],
    },
    CritterDef {
        id: 120100087, // White Skunk
        species: "Skunks",
        biome: "Modish Marsh",
        days: [PM, NA, PM, NA, PM, NA, PM],
        notes: &[],
    },
    CritterDef {
        id: 120100094, // Blue Sweet Bee
        species: "Sweet Bees",
        biome: "Hunny Falls",
        days: [ALL; 7],
        notes: &[Note::ClearWeather],
    },
    CritterDef {
        id: 120100093, // Pink Sweet Bee
        species: "Sweet Bees",
        biome: "Pixie Flats",
        days: [ALL; 7],
        notes: &[],
    },
    CritterDef {
        id: 120100092, // Sweet Bee
        species: "Sweet Bees",
        biome: "Sundae Shores",
        days: [hours(7, 20); 7],
        notes: &[],
    },
    CritterDef {
        id: 120100095, // White Sweet Bee
        species: "Sweet Bees",
        biome: "Hundred-Acre Fields",
        days: [hours(20, 24); 7],
        notes: &[],
    },
    CritterDef {
        id: 120100108, // Classic Hedgehog
        species: "Hedgehogs",
        biome: "Drowsybloom Acre",
        days: [ALL; 7],
        notes: &[],
    },
    CritterDef {
        id: 120100110, // Green Hedgehog
        species: "Hedgehogs",
        biome: "Gloommeadow",
        days: [AM; 7],
        notes: &[],
    },
    CritterDef {
        id: 120100111, // Orange Hedgehog
        species: "Hedgehogs",
        biome: "Nectar Apiary",
        days: [PM, NA, NA, PM, NA, NA, PM],
        notes: &[],
    },
    CritterDef {
        id: 120100109, // Yellow Hedgehog
        species: "Hedgehogs",
        biome: "Braveheart Grove",
        days: [PM; 7],
        notes: &[],
    },
];

// sort by in game order
fn species_rank(species: &str) -> u32 {
    CRITTER_DATA
        .iter()
        .position(|c| c.species == species)
        .unwrap_or(usize::MAX) as u32
}

#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, specta::Type)]
pub(crate) struct Schedule {
    start: u8,
    end: u8,
}

fn day_schedule(day: AvailableHours) -> Vec<Schedule> {
    let mut schedule = Vec::new();
    let mut hour: u8 = 0;
    while hour < 24 {
        if day >> hour & 1 == 0 {
            hour += 1;
            continue;
        }
        let start = hour;
        while hour < 24 && day >> hour & 1 == 1 {
            hour += 1;
        }
        schedule.push(Schedule { start, end: hour });
    }
    schedule
}

#[derive(Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Critter {
    pub item_id: u32,
    pub name: String,
    pub species: String,
    pub species_rank: u32,
    pub biome: String,
    pub notes: Vec<String>,
    pub schedule: Vec<Vec<Schedule>>,
    pub tamed: bool,
    #[specta(type = Option<Number>)]
    pub last_feeding_secs: Option<i64>,
}

pub(crate) fn collect(loaded: &super::LoadedSave) -> Result<Vec<Critter>, super::AppError> {
    let save = &loaded.contents;
    let storefront = &loaded.storefront;

    let name_map = super::game_data::cached_item_names(storefront)?;

    let unlocked_biomes = super::areas::unlocked_biomes(save);
    let completed_missions = save
        .pointer("/World/QuestInfo/MissionsCompleted")
        .and_then(|v| v.as_array());

    let companion_links = super::game_data::cached_companion_links(storefront)?;

    let tamed_companions: HashSet<u32> = save
        .pointer("/Player/Pets")
        .and_then(|v| v.as_array())
        .map(|pets| {
            pets.iter()
                .filter_map(|pet| Some(pet.get("PetItemID")?.as_u64()? as u32))
                .collect()
        })
        .unwrap_or_default();

    // recorded as UTC but actually local time
    let last_feeding_by_id: HashMap<u32, i64> = save
        .pointer("/World/Critters")
        .and_then(|v| v.as_array())
        .map(|critters| {
            critters
                .iter()
                .filter_map(|c| {
                    let id = c.get("CritterItemID")?.as_u64()? as u32;
                    let time_str = c.get("LastFeedingTime")?.as_str()?;
                    let ts = chrono::DateTime::parse_from_rfc3339(time_str)
                        .ok()?
                        .timestamp();
                    Some((id, ts))
                })
                .collect()
        })
        .unwrap_or_default();

    let critters = CRITTER_DATA
        .iter()
        .filter_map(|entry| {
            let biome_unlocked = unlocked_biomes
                .as_ref()
                .is_none_or(|unlocked| unlocked.contains(entry.biome));
            let quest_unlocked =
                entry
                    .notes
                    .iter()
                    .filter_map(Note::required_mission)
                    .all(|mission| {
                        completed_missions.is_none_or(|missions| {
                            missions.iter().any(|id| id.as_u64() == Some(mission))
                        })
                    });
            if !(biome_unlocked && quest_unlocked) {
                return None;
            }
            let companion_id = companion_links.get(&entry.id).copied();
            let name = companion_id
                .and_then(|id| name_map.get(&id))
                .or_else(|| name_map.get(&entry.id))
                .cloned()
                .unwrap_or_default();
            let schedule: Vec<Vec<Schedule>> =
                entry.days.iter().map(|&day| day_schedule(day)).collect();
            Some(Critter {
                item_id: entry.id,
                name,
                species: entry.species.to_string(),
                species_rank: species_rank(entry.species),
                biome: entry.biome.to_string(),
                notes: entry
                    .notes
                    .iter()
                    .filter(|n| n.required_mission().is_none())
                    .map(|n| n.as_str().to_string())
                    .collect(),
                schedule,
                tamed: companion_id.is_some_and(|id| tamed_companions.contains(&id)),
                last_feeding_secs: last_feeding_by_id.get(&entry.id).copied(),
            })
        })
        .collect();

    Ok(critters)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_critter_maps_to_a_known_biome() {
        let known = crate::areas::known_biome_names();
        for def in CRITTER_DATA {
            assert!(known.contains(def.biome), "{} -> {}", def.id, def.biome);
        }
    }

    #[test]
    fn mask_decodes_to_day_schedule() {
        assert!(day_schedule(NA).is_empty());
        assert_eq!(day_schedule(ALL), vec![Schedule { start: 0, end: 24 }]);
        assert_eq!(
            day_schedule(hours(7, 8) | hours(19, 20)),
            vec![
                Schedule { start: 7, end: 8 },
                Schedule { start: 19, end: 20 },
            ]
        );
        assert_eq!(
            day_schedule(hours(20, 24)),
            vec![Schedule { start: 20, end: 24 }]
        );
    }
}
