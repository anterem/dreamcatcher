use super::checklist::{self, ChecklistFacts};
use super::critters::{self, Critter};
use super::villagers::{self, Villager};
use specta_typescript::Number;

#[derive(Clone, serde::Serialize, specta::Type)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum Section<T> {
    Ok { data: T },
    Error { error: String },
}

impl<T> From<Result<T, super::AppError>> for Section<T> {
    fn from(result: Result<T, super::AppError>) -> Self {
        match result {
            Ok(data) => Section::Ok { data },
            Err(error) => Section::Error {
                error: error.to_string(),
            },
        }
    }
}

#[derive(Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    #[specta(type = Number)]
    tz_offset: i64,
    modified_secs: u32,
    critters: Section<Vec<Critter>>,
    villagers: Section<Vec<Villager>>,
    checklist: Section<ChecklistFacts>,
}

pub fn build(loaded: &super::LoadedSave) -> Snapshot {
    Snapshot {
        tz_offset: super::read_tz_offset(&loaded.contents),
        modified_secs: super::get_modified_secs(&loaded.path).unwrap_or(0),
        critters: critters::collect(loaded).into(),
        villagers: villagers::collect(loaded).into(),
        checklist: checklist::collect(loaded).into(),
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_snapshot(state: tauri::State<super::AppState>) -> Result<Snapshot, super::AppError> {
    let guard = state.save.lock().unwrap();
    let loaded = guard.as_ref().ok_or(super::AppError::NoSaveLoaded)?;
    Ok(build(loaded))
}
