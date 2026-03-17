use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub api_key: String,
    pub theme: String,
    pub courses_dir: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            theme: "dark".to_string(),
            courses_dir: default_courses_dir(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CourseSummary {
    pub id: String,
    pub title: String,
    pub description: String,
    pub total_lessons: usize,
    pub completed_lessons: usize,
}

fn default_courses_dir() -> String {
    dirs::document_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("RoosterCourses")
        .to_string_lossy()
        .to_string()
}

fn config_path(app: &tauri::AppHandle) -> PathBuf {
    let data_dir = app
        .path()
        .app_data_dir()
        .expect("failed to get app data dir");
    fs::create_dir_all(&data_dir).ok();
    data_dir.join("config.json")
}

fn read_course_manifest(course_dir: &Path) -> Option<CourseSummary> {
    let manifest_path = course_dir.join("course.yaml");
    let content = fs::read_to_string(&manifest_path).ok()?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content).ok()?;

    let title = yaml["title"].as_str().unwrap_or("Untitled").to_string();
    let description = yaml["description"].as_str().unwrap_or("").to_string();

    let lessons = yaml["lessons"].as_sequence().map(|s| s.len()).unwrap_or(0);
    let completed = yaml["lessons"]
        .as_sequence()
        .map(|seq| {
            seq.iter()
                .filter(|l| l["completed"].as_bool().unwrap_or(false))
                .count()
        })
        .unwrap_or(0);

    Some(CourseSummary {
        id: course_dir
            .file_name()?
            .to_string_lossy()
            .to_string(),
        title,
        description,
        total_lessons: lessons,
        completed_lessons: completed,
    })
}

#[tauri::command]
fn load_config(app: tauri::AppHandle) -> AppConfig {
    let path = config_path(&app);
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

#[tauri::command]
fn save_config(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    let path = config_path(&app);
    let content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())?;

    if !config.courses_dir.is_empty() {
        fs::create_dir_all(&config.courses_dir).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
fn get_default_courses_dir() -> String {
    default_courses_dir()
}

#[tauri::command]
fn list_courses(app: tauri::AppHandle) -> Vec<CourseSummary> {
    let config = load_config(app);
    let courses_dir = Path::new(&config.courses_dir);

    if !courses_dir.exists() {
        return vec![];
    }

    let mut courses = Vec::new();
    if let Ok(entries) = fs::read_dir(courses_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(summary) = read_course_manifest(&path) {
                    courses.push(summary);
                }
            }
        }
    }
    courses.sort_by(|a, b| a.title.cmp(&b.title));
    courses
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            load_config,
            save_config,
            get_default_courses_dir,
            list_courses,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
