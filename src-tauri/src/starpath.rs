use super::LoadedSave;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct StarPath {
    name: String,
    duties: Vec<StarPathDuty>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct StarPathDuty {
    description: String,
    progress: u32,
    target: u32,
}

impl StarPathDuty {
    fn from_task(description: &str, task: &Value) -> Self {
        let (progress, target) = task_progress(task);
        Self {
            description: description.trim_end_matches(['.', '!']).to_string(),
            progress,
            target,
        }
    }
}

pub(crate) fn collect(loaded: &LoadedSave) -> Vec<StarPath> {
    let Ok(labels) = super::game_data::cached_liveops_labels(&loaded.storefront) else {
        return Vec::new();
    };

    let now_secs = chrono::Utc::now().timestamp();
    let mut star_paths = Vec::new();
    if let Some(entry) = loaded
        .contents
        .pointer("/Player/BattlePassStates/FtueProgress")
        && !ftue_mission_completed(&loaded.contents)
        && let Ok(task_description_keys) =
            super::game_data::cached_ftue_task_loc_keys(&loaded.storefront)
    {
        star_paths.extend(collect_star_path(
            "StarPathFTUE",
            entry,
            &labels,
            &task_description_keys,
            now_secs,
        ));
    }
    if let Some(progress) = loaded
        .contents
        .pointer("/Player/BattlePassStates/Progress")
        .and_then(|v| v.as_object())
    {
        for (code, entry) in progress {
            let task_description_keys = seasonal_task_description_keys(&labels, code);
            star_paths.extend(collect_star_path(
                code,
                entry,
                &labels,
                &task_description_keys,
                now_secs,
            ));
        }
    }
    star_paths
}

// task numbers can have gaps, e.g. Seafaring2026 uses Task02 to Task55 for 25 tasks total
fn seasonal_task_description_keys(labels: &HashMap<String, String>, code: &str) -> Vec<String> {
    let prefix = format!("{code}_Task");
    let mut keys: Vec<(u32, String)> = labels
        .keys()
        .filter_map(|key| {
            let number = key
                .strip_prefix(&prefix)?
                .strip_suffix("_Description")?
                .parse()
                .ok()?;
            Some((number, key.clone()))
        })
        .collect();
    keys.sort_unstable_by_key(|(number, _)| *number);
    keys.into_iter().map(|(_, key)| key).collect()
}

fn collect_star_path(
    code: &str,
    entry: &Value,
    labels: &HashMap<String, String>,
    task_description_keys: &[String],
    now_secs: i64,
) -> Option<StarPath> {
    if !is_current(entry, now_secs) {
        return None;
    }
    let name = labels.get(&format!("{code}_Title"))?.clone();

    let mut duties = Vec::new();
    for (index, task) in ongoing_tasks(entry.get("TasksProgress")) {
        let description = task_description_keys
            .get(index)
            .and_then(|key| labels.get(key));
        if let Some(description) = description {
            duties.push(StarPathDuty::from_task(description, task));
        }
    }
    if let Some(groups) = entry.get("WeeklyTasksProgress").and_then(|v| v.as_array()) {
        for (week_index, group) in groups.iter().enumerate() {
            for (index, task) in ongoing_tasks(group.get("TasksProgress")) {
                if let Some(description) = weekly_duty_description(labels, code, week_index, index)
                {
                    duties.push(StarPathDuty::from_task(description, task));
                }
            }
        }
    }
    (!duties.is_empty()).then_some(StarPath { name, duties })
}

// last duty of FTUE is always incomplete, check mission status instead
const STAR_PATH_FTUE_MISSION_ID: u64 = 2070001657;

fn ftue_mission_completed(contents: &Value) -> bool {
    contents
        .pointer("/World/QuestInfo/MissionsCompleted")
        .and_then(|v| v.as_array())
        .is_some_and(|missions| {
            missions
                .iter()
                .any(|m| m.as_u64() == Some(STAR_PATH_FTUE_MISSION_ID))
        })
}

fn is_current(entry: &Value, now_secs: i64) -> bool {
    entry
        .get("EndDate")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .is_some_and(|end| end.timestamp() > now_secs)
}

fn weekly_duty_description<'a>(
    labels: &'a HashMap<String, String>,
    code: &str,
    week_index: usize,
    task_index: usize,
) -> Option<&'a str> {
    let week = week_index + 1;
    let task = format!("Task{:02}", task_index + 1);
    labels
        .get(&format!("{code}_Week{week:02}{task}_Description"))
        // StarPathFTUE uses Week1 instead of Week01 like other star paths
        .or_else(|| labels.get(&format!("{code}_Week{week}{task}_Description")))
        .map(String::as_str)
}

fn ongoing_tasks(tasks: Option<&Value>) -> impl Iterator<Item = (usize, &Value)> {
    tasks
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .enumerate()
        .filter(|(_, task)| {
            task.get("TaskState").and_then(|v| v.as_str()) == Some("BattlePassTaskState_OnGoing")
        })
}

fn task_progress(task: &Value) -> (u32, u32) {
    let objective = task.get("Objective");
    let amount = |field: &str| {
        objective
            .and_then(|v| v.get(field))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32
    };
    (amount("CurAmount"), amount("TargetAmount"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const ONGOING: &str = "BattlePassTaskState_OnGoing";
    const CLAIMED: &str = "BattlePassTaskState_CurrencyClaimed";
    const NOT_STARTED: &str = "BattlePassTaskState_NotStarted";

    fn task(state: &str, progress: u32, target: u32) -> Value {
        json!({ "Objective": { "CurAmount": progress, "TargetAmount": target }, "TaskState": state })
    }

    fn labels(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|&(key, text)| (key.to_string(), text.to_string()))
            .collect()
    }

    fn current_star_path(tasks: Value) -> Value {
        json!({ "EndDate": "2026-12-09T14:00:00Z", "TasksProgress": tasks })
    }

    #[test]
    fn only_ongoing_tasks_are_available() {
        let tasks = json!([
            task(CLAIMED, 3, 3),
            task(ONGOING, 4, 20),
            task(NOT_STARTED, 0, 5),
        ]);

        assert_eq!(
            ongoing_tasks(Some(&tasks)).collect::<Vec<_>>(),
            [(1, &task(ONGOING, 4, 20))]
        );
    }

    #[test]
    fn ongoing_duties_grouped_by_star_path() {
        let current = current_star_path(json!([task(ONGOING, 4, 20), task(NOT_STARTED, 0, 5),]));
        let labels = labels(&[
            ("PopCity2026_Title", "Pop City Star Path"),
            ("PopCity2026_Task01_Description", "Catch some fish."),
        ]);
        let task_description_keys = vec!["PopCity2026_Task01_Description".to_string()];

        assert_eq!(
            collect_star_path("PopCity2026", &current, &labels, &task_description_keys, 0).unwrap(),
            StarPath {
                name: "Pop City Star Path".to_string(),
                duties: vec![StarPathDuty {
                    description: "Catch some fish".to_string(),
                    progress: 4,
                    target: 20,
                }],
            }
        );
    }

    #[test]
    fn ftue_duties_use_loc_keys_by_index() {
        let current = current_star_path(json!([
            task(ONGOING, 0, 5),
            task(CLAIMED, 1, 1),
            task(NOT_STARTED, 0, 1),
        ]));
        let labels = labels(&[
            ("StarPathFTUE_Title", "Astronomer's Journey Star Path"),
            ("StarPathFTUE_Task02_Description", "Whip up a 2-star meal."),
            ("StarPathFTUE_Task03_Description", "Go fish!"),
        ]);
        let task_description_keys = vec![
            "StarPathFTUE_Task03_Description".to_string(),
            "StarPathFTUE_Task02_Description".to_string(),
        ];

        assert_eq!(
            collect_star_path("StarPathFTUE", &current, &labels, &task_description_keys, 0)
                .unwrap()
                .duties,
            [StarPathDuty {
                description: "Go fish".to_string(),
                progress: 0,
                target: 5,
            }]
        );
    }

    #[test]
    fn weekly_keys_fall_back_to_unpadded_week() {
        let labels = labels(&[("StarPathFTUE_Week1Task01_Description", "Plant vegetables")]);

        assert_eq!(
            weekly_duty_description(&labels, "StarPathFTUE", 0, 0),
            Some("Plant vegetables")
        );
    }
}
