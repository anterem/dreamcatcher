use std::{
    collections::HashMap,
    fs::read,
    io::{Cursor, Read},
    ops::Range,
    path::PathBuf,
    sync::{Arc, Mutex},
};
#[cfg(windows)]
use winreg::{RegKey, enums};
use zip::ZipArchive;

#[cfg(not(windows))]
pub(crate) const STEAM_APP_ID: u32 = 1401590;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum Storefront {
    Steam,
    Epic,
    Microsoft,
}

impl std::str::FromStr for Storefront {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "steam" => Ok(Self::Steam),
            "epic" => Ok(Self::Epic),
            "microsoft" => Ok(Self::Microsoft),
            _ => Err(()),
        }
    }
}

// don't crash on unexpected format
fn read_varint(data: &[u8], pos: &mut usize) -> Option<u64> {
    let mut result = 0u64;
    let mut shift = 0;
    loop {
        let byte = *data.get(*pos)?;
        *pos += 1;
        result |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            return Some(result);
        }
        shift += 7;
        if shift >= 64 {
            return None;
        }
    }
}

fn read_bytes<'a>(data: &'a [u8], pos: &mut usize) -> Option<&'a [u8]> {
    let len = read_varint(data, pos)? as usize;
    let end = (*pos).checked_add(len)?;
    let slice = data.get(*pos..end)?;
    *pos = end;
    Some(slice)
}

fn parse_vdf_paths(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|line| line.trim().starts_with(r#""path""#))
        .filter_map(|line| {
            line.split('\t')
                .last()
                .map(|s| s.trim().trim_matches('"').to_string())
        })
        .collect()
}

fn parse_acf_field(content: &str, field: &str) -> Option<String> {
    let target = format!(r#""{}""#, field);
    content
        .lines()
        .find(|line| line.trim().starts_with(&target))
        .and_then(|line| {
            line.split('\t')
                .last()
                .map(|s| s.trim().trim_matches('"').to_string())
        })
}

#[cfg(windows)]
fn steam_install_dir() -> Option<PathBuf> {
    let hkcu = RegKey::predef(enums::HKEY_CURRENT_USER);
    let steam_key = hkcu.open_subkey(r"Software\Valve\Steam").ok().or_else(|| {
        RegKey::predef(enums::HKEY_LOCAL_MACHINE)
            .open_subkey(r"Software\Valve\Steam")
            .ok()
    })?;
    let steam_path: String = steam_key.get_value("SteamPath").ok()?;
    Some(PathBuf::from(steam_path))
}

// steam deck
#[cfg(not(windows))]
fn steam_install_dir() -> Option<PathBuf> {
    let home = PathBuf::from(std::env::var_os("HOME")?);
    [
        home.join(".steam/steam"),
        home.join(".local/share/Steam"),
        home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
    ]
    .into_iter()
    .find(|p| p.exists())
}

pub(crate) fn steam_libraries() -> Vec<PathBuf> {
    let Some(steam_dir) = steam_install_dir() else {
        return vec![];
    };
    let vdf_content =
        std::fs::read_to_string(steam_dir.join("config/libraryfolders.vdf")).unwrap_or_default();
    let mut libraries: Vec<PathBuf> = parse_vdf_paths(&vdf_content)
        .into_iter()
        .map(PathBuf::from)
        .collect();
    if !libraries.contains(&steam_dir) {
        libraries.insert(0, steam_dir);
    }
    libraries
}

fn find_steam_streaming_assets() -> Option<PathBuf> {
    for library in steam_libraries() {
        let apps_dir = library.join("steamapps");
        let Ok(entries) = std::fs::read_dir(&apps_dir) else {
            continue;
        };

        for entry in entries {
            let Ok(entry) = entry else { continue };
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("appmanifest_") || !name.ends_with(".acf") {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(entry.path()) else {
                continue;
            };
            let Some(app_name) = parse_acf_field(&content, "name") else {
                continue;
            };
            if app_name == "Disney Dreamlight Valley" {
                let Some(installdir) = parse_acf_field(&content, "installdir") else {
                    continue;
                };
                let streaming_assets = apps_dir
                    .join("common")
                    .join(&installdir)
                    .join("ddv_Data")
                    .join("StreamingAssets");
                if streaming_assets.exists() {
                    return Some(streaming_assets);
                }
            }
        }
    }

    None
}

#[cfg(windows)]
fn find_epic_streaming_assets() -> Option<PathBuf> {
    let programdata = std::env::var("PROGRAMDATA").ok()?;
    let manifests_dir = PathBuf::from(&programdata)
        .join("Epic")
        .join("EpicGamesLauncher")
        .join("Data")
        .join("Manifests");
    let Ok(entries) = std::fs::read_dir(&manifests_dir) else {
        return None;
    };

    for entry in entries {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("item") {
            continue;
        }

        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) else {
            continue;
        };
        if manifest.get("DisplayName").and_then(|v| v.as_str()) == Some("Disney Dreamlight Valley")
        {
            if let Some(install_location) = manifest.get("InstallLocation").and_then(|v| v.as_str())
            {
                let streaming_assets = PathBuf::from(install_location)
                    .join("ddv_Data")
                    .join("StreamingAssets");
                if streaming_assets.exists() {
                    return Some(streaming_assets);
                }
            }
        }
    }

    None
}

#[cfg(windows)]
fn find_ms_streaming_assets() -> Option<PathBuf> {
    let hklm = RegKey::predef(enums::HKEY_LOCAL_MACHINE);
    let package_key = hklm
        .open_subkey(r"SOFTWARE\Microsoft\GamingServices\PackageRepository\Package")
        .ok()?;

    let app_ids: Vec<String> = package_key
        .enum_values()
        .filter_map(|r| r.ok().map(|(name, _)| name))
        .collect();

    for app_id in &app_ids {
        let packages_key_path = format!(
            r"SOFTWARE\Classes\Local Settings\Software\Microsoft\Windows\CurrentVersion\AppModel\PackageRepository\Packages\{}",
            app_id
        );
        if let Ok(packages_key) = hklm.open_subkey(&packages_key_path) {
            if let Ok(path) = packages_key.get_value::<String, _>("Path") {
                let streaming_assets = PathBuf::from(path).join("ddv_Data").join("StreamingAssets");
                if streaming_assets.exists() {
                    return Some(streaming_assets);
                }
            }
        }
    }

    None
}

fn find_streaming_assets(storefront: &Storefront) -> Option<PathBuf> {
    match storefront {
        Storefront::Steam => find_steam_streaming_assets(),
        #[cfg(windows)]
        Storefront::Epic => find_epic_streaming_assets(),
        #[cfg(windows)]
        Storefront::Microsoft => find_ms_streaming_assets(),
        #[cfg(not(windows))]
        _ => None,
    }
}

// protobuf records with unknown wrapper - have to scan for id and key instead of parsing
fn resolve_display_names(
    bytes: &[u8],
    id_range: Range<u32>,
    loc_map: &HashMap<String, String>,
) -> HashMap<u32, String> {
    let mut names = HashMap::new();
    let mut pos = 0;

    while pos < bytes.len() {
        if bytes[pos] != 0x08 {
            pos += 1;
            continue;
        }
        let id_start = pos + 1;

        // use scratch cursor to check for false positives
        let mut scan = id_start;
        if let Some(id) = read_varint(bytes, &mut scan) {
            if id_range.contains(&(id as u32)) && bytes.get(scan) == Some(&0x12) {
                scan += 1;
                if let Some(data) = read_bytes(bytes, &mut scan) {
                    if let Ok(key) = std::str::from_utf8(data) {
                        // locbin key has two forms
                        // "Fish!Bass" -> "Fish!Bass_DisplayName" - field 2
                        //  "ActivityItem.Fish!Expansion01!Piranha_DisplayName" - field 3
                        if let Some(name) = loc_map.get(&format!("{key}_DisplayName")) {
                            names.insert(id as u32, name.clone());
                            pos = scan;
                            continue;
                        }
                        let mut f3 = scan;
                        if bytes.get(f3) == Some(&0x1a) {
                            f3 += 1;
                            if let Some(full) = read_bytes(bytes, &mut f3) {
                                if let Some(loc_key) = std::str::from_utf8(full)
                                    .ok()
                                    .and_then(|s| s.split_once('.'))
                                {
                                    if let Some(name) = loc_map.get(loc_key.1) {
                                        names.insert(id as u32, name.clone());
                                        pos = f3;
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        pos = id_start;
    }

    names
}

fn resolve_companion_links(bytes: &[u8], wild_range: Range<u32>) -> HashMap<u32, u32> {
    let mut links = HashMap::new();
    let mut pos = 0;

    while pos < bytes.len() {
        if bytes[pos] != 0x08 {
            pos += 1;
            continue;
        }
        let mut scan = pos + 1;
        if let Some(id) = read_varint(bytes, &mut scan) {
            if wild_range.contains(&(id as u32)) && bytes.get(scan) == Some(&0x12) {
                if let Some(companion) = read_companion_field(bytes, scan) {
                    links.insert(id as u32, companion);
                }
            }
        }
        pos += 1;
    }

    links
}

// wild critter links to associated companion at field 23
fn read_companion_field(bytes: &[u8], mut pos: usize) -> Option<u32> {
    const COMPANION_RANGE: Range<u32> = 120_000_000..120_100_000;
    while pos + 1 < bytes.len() {
        if bytes[pos] == 0x08 {
            let mut scan = pos + 1;
            if let Some(next_id) = read_varint(bytes, &mut scan) {
                if (120_000_000..120_300_000).contains(&(next_id as u32))
                    && bytes.get(scan) == Some(&0x12)
                {
                    return None;
                }
            }
        }
        if bytes[pos] == 0xB8 && bytes.get(pos + 1) == Some(&0x01) {
            let mut scan = pos + 2;
            if let Some(companion) = read_varint(bytes, &mut scan) {
                if COMPANION_RANGE.contains(&(companion as u32)) {
                    return Some(companion as u32);
                }
            }
        }
        pos += 1;
    }
    None
}

fn parse_loc_map(bytes: &[u8]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut pos = 0;

    while pos < bytes.len() {
        let Some(outer_tag) = read_varint(bytes, &mut pos) else {
            break;
        };
        if outer_tag != 0x0a {
            if skip_field(bytes, &mut pos, outer_tag).is_none() {
                break;
            }
            continue;
        }

        let Some(submessage) = read_bytes(bytes, &mut pos) else {
            break;
        };
        let mut sub_pos = 0;
        let mut key = None;
        let mut name = None;

        while sub_pos < submessage.len() {
            let Some(inner_tag) = read_varint(submessage, &mut sub_pos) else {
                break;
            };
            match inner_tag {
                0x0a => {
                    let Some(data) = read_bytes(submessage, &mut sub_pos) else {
                        break;
                    };
                    key = Some(std::str::from_utf8(data).unwrap_or_default().to_string());
                }
                0x12 => {
                    let Some(data) = read_bytes(submessage, &mut sub_pos) else {
                        break;
                    };
                    name = Some(std::str::from_utf8(data).unwrap_or_default().to_string());
                }
                _ => {
                    if skip_field(submessage, &mut sub_pos, inner_tag).is_none() {
                        break;
                    }
                }
            }
        }

        if let (Some(k), Some(n)) = (key, name) {
            map.insert(k, n);
        }
    }

    map
}

// protobuf wire types
fn skip_field(data: &[u8], pos: &mut usize, tag: u64) -> Option<()> {
    match tag & 0x07 {
        0 => read_varint(data, pos).map(|_| ()),
        1 => {
            *pos += 8;
            Some(())
        }
        2 => read_bytes(data, pos).map(|_| ()),
        5 => {
            *pos += 4;
            Some(())
        }
        _ => None,
    }
}

fn read_loc_entry(storefront: &Storefront, entry_name: &str) -> Result<Vec<u8>, super::AppError> {
    let streaming_assets = find_streaming_assets(storefront)
        .ok_or_else(|| super::AppError::NotFound("Could not locate game files".to_string()))?;
    let zip_bytes = read(
        streaming_assets
            .join("Localization")
            .join("LocDB_en-US.zip"),
    )?;
    let mut archive = ZipArchive::new(Cursor::new(zip_bytes))
        .map_err(|e| super::AppError::Parse(e.to_string()))?;
    let mut entry = archive.by_name(entry_name).map_err(|_| {
        super::AppError::NotFound(format!("Could not find {} in LocDB archive", entry_name))
    })?;
    let mut loc_bytes = Vec::new();
    entry
        .read_to_end(&mut loc_bytes)
        .map_err(|e| super::AppError::Parse(e.to_string()))?;
    Ok(loc_bytes)
}

fn load_item_names(storefront: &Storefront) -> Result<HashMap<u32, String>, super::AppError> {
    let streaming_assets = find_streaming_assets(storefront)
        .ok_or_else(|| super::AppError::NotFound("Could not locate game files".to_string()))?;

    // disjoint id ranges, so merging every type into one map is collision-free
    let types = [
        ("Companion", 120_000_000..120_300_000),
        ("Character", 10_000_000..10_100_000),
        ("ActivityItem", 30_000_000..32_000_000),
        ("Furniture", 40_000_000..42_000_000),
        ("Clothing", 50_000_000..52_000_000),
        ("Trimming", 160_000_000..161_200_000),
    ];
    let mut names = HashMap::new();

    for (t, id_range) in types {
        let item_bytes = read(
            streaming_assets
                .join("itemlist")
                .join(format!("{}.json", t)),
        )?;
        let loc_map = parse_loc_map(&read_loc_entry(storefront, &format!("{}.locbin", t))?);
        names.extend(resolve_display_names(&item_bytes, id_range, &loc_map));
    }

    Ok(names)
}

static CACHE: Mutex<Option<(Storefront, Arc<HashMap<u32, String>>)>> = Mutex::new(None);

// cache item names to avoid re-parsing game files
pub fn cached_item_names(
    storefront: &Storefront,
) -> Result<Arc<HashMap<u32, String>>, super::AppError> {
    let mut cache = CACHE.lock().unwrap();
    if let Some((cached_storefront, names)) = cache.as_ref() {
        if cached_storefront == storefront {
            return Ok(names.clone());
        }
    }

    let names = Arc::new(load_item_names(storefront)?);
    *cache = Some((storefront.clone(), names.clone()));
    Ok(names)
}

fn load_companion_links(storefront: &Storefront) -> Result<HashMap<u32, u32>, super::AppError> {
    let streaming_assets = find_streaming_assets(storefront)
        .ok_or_else(|| super::AppError::NotFound("Could not locate game files".to_string()))?;
    let item_bytes = read(streaming_assets.join("itemlist").join("Companion.json"))?;
    Ok(resolve_companion_links(
        &item_bytes,
        120_100_000..120_200_000,
    ))
}

static COMPANION_CACHE: Mutex<Option<(Storefront, Arc<HashMap<u32, u32>>)>> = Mutex::new(None);

pub fn cached_companion_links(
    storefront: &Storefront,
) -> Result<Arc<HashMap<u32, u32>>, super::AppError> {
    let mut cache = COMPANION_CACHE.lock().unwrap();
    if let Some((cached_storefront, links)) = cache.as_ref() {
        if cached_storefront == storefront {
            return Ok(links.clone());
        }
    }

    let links = Arc::new(load_companion_links(storefront)?);
    *cache = Some((storefront.clone(), links.clone()));
    Ok(links)
}

static MENU_CACHE: Mutex<Option<(Storefront, Arc<HashMap<String, String>>)>> = Mutex::new(None);

pub(crate) fn cached_menu_labels(
    storefront: &Storefront,
) -> Result<Arc<HashMap<String, String>>, super::AppError> {
    let mut cache = MENU_CACHE.lock().unwrap();
    if let Some((cached_storefront, labels)) = cache.as_ref() {
        if cached_storefront == storefront {
            return Ok(labels.clone());
        }
    }

    let labels = Arc::new(parse_loc_map(&read_loc_entry(storefront, "menu.locbin")?));
    *cache = Some((storefront.clone(), labels.clone()));
    Ok(labels)
}

// display names for game ids: companions, characters, and items alike
#[tauri::command]
#[specta::specta]
pub fn get_display_names(storefront: Storefront) -> Result<HashMap<u32, String>, super::AppError> {
    Ok((*cached_item_names(&storefront)?).clone())
}
