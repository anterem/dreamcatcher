use cipher::{BlockModeDecrypt, KeyInit, block_padding::Pkcs7};
#[cfg(windows)]
use known_folders::{KnownFolder, get_known_folder_path};
use notify_debouncer_full::{
    DebounceEventResult, Debouncer, RecommendedCache, new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode},
};
use std::{
    ffi::OsStr,
    fs::read,
    io::{Cursor, Read},
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

mod areas;
mod checklist;
mod critters;
mod game_data;
mod resets;
mod snapshot;
mod starpath;
mod villagers;

const KEY: [u8; 32] =
    hex_literal::hex!("62357168683873614a38556c444a557a545a5864325467366d626f3857386e35");

const SAVE_FILE_NAME: &str = "profile.json";

const WATCH_DEBOUNCE: Duration = Duration::from_millis(400);
// retry in case the read lands mid-write and fails
const READ_RETRIES: u32 = 5;
const RETRY_DELAY: Duration = Duration::from_millis(120);

type SaveWatcher = Debouncer<RecommendedWatcher, RecommendedCache>;

#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("{0}")]
    NotFound(String),
    #[error("No save file loaded")]
    NoSaveLoaded,
}

pub struct LoadedSave {
    pub path: PathBuf,
    pub contents: serde_json::Value,
    pub storefront: game_data::Storefront,
}

pub struct AppState {
    pub save: Mutex<Option<LoadedSave>>,
    pub watcher: Mutex<Option<SaveWatcher>>,
}

#[derive(Clone, serde::Serialize, specta::Type, tauri_specta::Event)]
pub struct SaveChanged(snapshot::Snapshot);

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

#[derive(serde::Serialize, specta::Type)]
pub struct SaveFile {
    path: PathBuf,
    storefront: game_data::Storefront,
    #[serde(rename = "lastModified")]
    modified_secs: u32,
}

// in the save as e.g. "-25200s"
pub fn read_tz_offset(save: &serde_json::Value) -> i64 {
    save.pointer("/World/TimeZoneOffset")
        .and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_str()?.trim_end_matches('s').parse::<i64>().ok())
        })
        .unwrap_or(0)
}

fn decrypt_save_file(path: &Path) -> Result<serde_json::Value, AppError> {
    let ciphertext = read(path)?;

    let decryptor = ecb::Decryptor::<aes::Aes256>::new(&KEY.into());

    let plaintext = decryptor
        .decrypt_padded_vec::<Pkcs7>(&ciphertext)
        .map_err(|e| AppError::Parse(e.to_string()))?;

    let cursor = Cursor::new(plaintext);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| AppError::Parse(e.to_string()))?;
    let mut save_file = archive
        .by_index(0)
        .map_err(|e| AppError::Parse(e.to_string()))?;

    let mut contents = String::new();
    save_file
        .read_to_string(&mut contents)
        .map_err(|e| AppError::Parse(e.to_string()))?;

    serde_json::from_str(&contents).map_err(|e| AppError::Parse(e.to_string()))
}

fn decrypt_with_retry(path: &Path) -> Result<serde_json::Value, AppError> {
    let mut last_err = AppError::NoSaveLoaded;
    for attempt in 0..READ_RETRIES {
        match decrypt_save_file(path) {
            Ok(value) => return Ok(value),
            Err(err) => {
                last_err = err;
                if attempt + 1 < READ_RETRIES {
                    std::thread::sleep(RETRY_DELAY);
                }
            }
        }
    }
    Err(last_err)
}

fn store_and_emit(app: &AppHandle, loaded: LoadedSave) {
    let snapshot = snapshot::build(&loaded);
    *app.state::<AppState>().save.lock().unwrap() = Some(loaded);
    let _ = SaveChanged(snapshot).emit(app);
}

fn reload(app: &AppHandle) {
    let (path, storefront) = {
        let state = app.state::<AppState>();
        let guard = state.save.lock().unwrap();
        match guard.as_ref() {
            Some(loaded) => (loaded.path.clone(), loaded.storefront.clone()),
            None => return,
        }
    };
    let Ok(contents) = decrypt_with_retry(&path) else {
        return;
    };
    store_and_emit(
        app,
        LoadedSave {
            path,
            contents,
            storefront,
        },
    );
}

fn watch_save(app: &AppHandle, path: PathBuf) -> Result<(), AppError> {
    let folder = path
        .parent()
        .ok_or_else(|| AppError::Io("save file has no parent folder".to_string()))?
        .to_path_buf();

    let handler_app = app.clone();
    let mut debouncer = new_debouncer(WATCH_DEBOUNCE, None, move |result: DebounceEventResult| {
        let Ok(events) = result else {
            return;
        };
        let touched = events.iter().any(|event| {
            event
                .paths
                .iter()
                .any(|p| p.file_name() == Some(OsStr::new(SAVE_FILE_NAME)))
        });
        if touched {
            reload(&handler_app);
        }
    })
    .map_err(|e| AppError::Io(e.to_string()))?;

    debouncer
        .watch(&folder, RecursiveMode::NonRecursive)
        .map_err(|e| AppError::Io(e.to_string()))?;

    *app.state::<AppState>().watcher.lock().unwrap() = Some(debouncer);
    Ok(())
}

#[tauri::command]
#[specta::specta]
fn load_save_file(
    app: AppHandle,
    path: PathBuf,
    storefront: game_data::Storefront,
) -> Result<(), String> {
    // stringify here so the frontend just displays errors (see Section::Error)
    let contents = decrypt_save_file(&path).map_err(|e| e.to_string())?;
    store_and_emit(
        &app,
        LoadedSave {
            path: path.clone(),
            contents,
            storefront,
        },
    );
    watch_save(&app, path).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(windows)]
fn get_game_folder() -> Option<PathBuf> {
    Some(
        get_known_folder_path(KnownFolder::LocalAppDataLow)
            .expect("LocalAppDataLow exists on Windows")
            .join("Gameloft")
            .join("Disney Dreamlight Valley"),
    )
}

// on linux (steam deck) the game runs in a proton prefix
#[cfg(not(windows))]
fn get_game_folder() -> Option<PathBuf> {
    game_data::steam_libraries()
        .into_iter()
        .map(|lib| {
            lib.join(format!(
                "steamapps/compatdata/{}/pfx/drive_c/users/steamuser/AppData/LocalLow/Gameloft/Disney Dreamlight Valley",
                game_data::STEAM_APP_ID
            ))
        })
        .find(|p| p.exists())
}

pub(crate) fn get_modified_secs(path: &Path) -> Option<u32> {
    let meta = path.metadata().ok()?;
    let modified = meta.modified().ok()?;
    let duration = modified.duration_since(UNIX_EPOCH).ok()?;
    Some(duration.as_secs() as u32)
}

#[tauri::command]
#[specta::specta]
fn get_save_files() -> Result<Vec<SaveFile>, String> {
    let Some(game_path) = get_game_folder() else {
        return Ok(vec![]);
    };

    let entries = match std::fs::read_dir(&game_path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        result => result.map_err(|e| AppError::from(e).to_string())?,
    };

    let save_files: Vec<SaveFile> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_ok_and(|ft| ft.is_dir()))
        .filter_map(|entry| {
            let save_file_path = entry.path().join(SAVE_FILE_NAME);
            if !save_file_path.try_exists().unwrap_or_default() {
                return None;
            }
            let storefront = entry
                .file_name()
                .to_string_lossy()
                .split('_')
                .next()
                .unwrap()
                .parse()
                .ok()?;
            let modified_secs = get_modified_secs(&save_file_path).unwrap_or(0);

            Some(SaveFile {
                path: save_file_path,
                storefront,
                modified_secs,
            })
        })
        .collect();

    Ok(save_files)
}

fn deck_gaming_mode() -> bool {
    std::env::var("SteamOS").as_deref() == Ok("1")
        && std::env::var("SteamGamepadUI").as_deref() == Ok("1")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            get_save_files,
            load_save_file,
            game_data::get_display_names,
            snapshot::get_snapshot
        ])
        .events(tauri_specta::collect_events![SaveChanged]);

    #[cfg(debug_assertions)]
    specta_builder
        .export(
            specta_typescript::Typescript::default(),
            "../src/lib/bindings.ts",
        )
        .expect("failed to export ts bindings");

    tauri::Builder::default()
        .manage(AppState {
            save: Mutex::new(None),
            watcher: Mutex::new(None),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            specta_builder.mount_events(app);
            if deck_gaming_mode() {
                if let Some(window) = app.get_webview_window("main") {
                    window.set_fullscreen(true)?;
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
