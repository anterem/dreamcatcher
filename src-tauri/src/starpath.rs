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
    duty: String,
    progress: u32,
    target: u32,
}

impl StarPathDuty {
    fn from_task(description: &str, task: &Value) -> Self {
        let (progress, target) = duty_progress(task);
        Self {
            duty: description
                .strip_suffix('.')
                .unwrap_or(description)
                .to_string(),
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
    {
        star_paths.extend(collect_star_path("StarPathFTUE", entry, &labels, now_secs));
    }
    if let Some(progress) = loaded
        .contents
        .pointer("/Player/BattlePassStates/Progress")
        .and_then(|v| v.as_object())
    {
        for (code, entry) in progress {
            star_paths.extend(collect_star_path(code, entry, &labels, now_secs));
        }
    }
    star_paths
}

fn collect_star_path(
    code: &str,
    entry: &Value,
    labels: &HashMap<String, String>,
    now_secs: i64,
) -> Option<StarPath> {
    if !is_current(entry, now_secs) {
        return None;
    }
    let name = labels.get(&format!("{code}_Title"))?.clone();

    let mut duties = Vec::new();
    for (n, task) in ongoing_tasks(entry.get("TasksProgress")) {
        if let Some(description) = duty_description(labels, code, None, n) {
            duties.push(StarPathDuty::from_task(description, task));
        }
    }
    if let Some(groups) = entry.get("WeeklyTasksProgress").and_then(|v| v.as_array()) {
        for (w, group) in groups.iter().enumerate() {
            let week = (w + 1) as u32;
            for (n, task) in ongoing_tasks(group.get("TasksProgress")) {
                if let Some(description) = duty_description(labels, code, Some(week), n) {
                    duties.push(StarPathDuty::from_task(description, task));
                }
            }
        }
    }
    (!duties.is_empty()).then_some(StarPath { name, duties })
}

fn is_current(entry: &Value, now_secs: i64) -> bool {
    entry
        .get("EndDate")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .is_some_and(|end| end.timestamp() > now_secs)
}

fn duty_description<'a>(
    labels: &'a HashMap<String, String>,
    code: &str,
    week: Option<u32>,
    task: u32,
) -> Option<&'a str> {
    let task = format!("Task{task:02}");
    let key = match week {
        Some(week) => format!("{code}_Week{week:02}{task}_Description"),
        None => format!("{code}_{task}_Description"),
    };
    labels
        .get(&key)
        // StarPathFTUE uses Week1 instead of Week01 like other star paths
        .or_else(|| {
            week.and_then(|week| labels.get(&format!("{code}_Week{week}{task}_Description")))
        })
        .map(String::as_str)
}

fn ongoing_tasks<'a>(tasks: Option<&'a Value>) -> impl Iterator<Item = (u32, &'a Value)> {
    tasks
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .enumerate()
        .filter(|(_, task)| {
            task.get("TaskState").and_then(|v| v.as_str()) == Some("BattlePassTaskState_OnGoing")
        })
        .map(|(i, task)| ((i + 1) as u32, task))
}

fn duty_progress(task: &Value) -> (u32, u32) {
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

    #[test]
    fn only_ongoing_tasks_are_available() {
        let tasks = json!([
            task(CLAIMED, 3, 3),
            task(ONGOING, 4, 20),
            task(NOT_STARTED, 0, 5),
        ]);

        let available: Vec<_> = ongoing_tasks(Some(&tasks)).collect();

        assert_eq!(available.len(), 1);
        assert_eq!(available[0].0, 2);
        assert_eq!(available[0].1, &task(ONGOING, 4, 20));
    }

    #[test]
    fn ongoing_duties_grouped_by_star_path() {
        let entry = json!({
            "EndDate": "2026-12-09T14:00:00Z",
            "TasksProgress": [
                task(ONGOING, 4, 20),
                task(NOT_STARTED, 0, 5),
            ],
        });
        let labels = labels(&[
            ("PopCity2026_Title", "Pop City Star Path"),
            ("PopCity2026_Task01_Description", "Catch some fish."),
        ]);

        let star_path = collect_star_path("PopCity2026", &entry, &labels, 0).unwrap();

        assert_eq!(star_path.name, "Pop City Star Path");
        assert_eq!(
            star_path.duties,
            vec![StarPathDuty {
                duty: "Catch some fish".to_string(),
                progress: 4,
                target: 20,
            }]
        );
    }
}
