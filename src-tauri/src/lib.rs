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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lesson {
    pub name: String,
    pub file: String,
    pub completed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CourseDetail {
    pub id: String,
    pub title: String,
    pub description: String,
    pub lessons: Vec<Lesson>,
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
fn config_exists(app: tauri::AppHandle) -> bool {
    config_path(&app).exists()
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

#[tauri::command]
fn get_course_detail(app: tauri::AppHandle, course_id: String) -> Result<CourseDetail, String> {
    let config = load_config(app);
    let course_dir = Path::new(&config.courses_dir).join(&course_id);
    let manifest = course_dir.join("course.yaml");

    let content = fs::read_to_string(&manifest).map_err(|e| e.to_string())?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content).map_err(|e| e.to_string())?;

    let title = yaml["title"].as_str().unwrap_or("Untitled").to_string();
    let description = yaml["description"].as_str().unwrap_or("").to_string();

    let lessons = yaml["lessons"]
        .as_sequence()
        .map(|seq| {
            seq.iter()
                .map(|l| Lesson {
                    name: l["name"].as_str().unwrap_or("").to_string(),
                    file: l["file"].as_str().unwrap_or("").to_string(),
                    completed: l["completed"].as_bool().unwrap_or(false),
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(CourseDetail {
        id: course_id,
        title,
        description,
        lessons,
    })
}

#[tauri::command]
fn get_lesson_content(
    app: tauri::AppHandle,
    course_id: String,
    file: String,
) -> Result<String, String> {
    let config = load_config(app);
    let path = Path::new(&config.courses_dir).join(&course_id).join(&file);
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn complete_lesson(
    app: tauri::AppHandle,
    course_id: String,
    lesson_file: String,
) -> Result<(), String> {
    let config = load_config(app);
    let manifest_path = Path::new(&config.courses_dir)
        .join(&course_id)
        .join("course.yaml");

    let content = fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
    let mut yaml: serde_yaml::Value =
        serde_yaml::from_str(&content).map_err(|e| e.to_string())?;

    if let Some(lessons) = yaml["lessons"].as_sequence_mut() {
        for lesson in lessons.iter_mut() {
            if lesson["file"].as_str() == Some(lesson_file.as_str()) {
                lesson["completed"] = serde_yaml::Value::Bool(true);
            }
        }
    }

    let updated = serde_yaml::to_string(&yaml).map_err(|e| e.to_string())?;
    fs::write(&manifest_path, updated).map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chat {
    pub id: String,
    pub title: String,
    pub created_at: u64,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatSummary {
    pub id: String,
    pub title: String,
    pub created_at: u64,
    pub message_count: usize,
}

fn chats_dir(courses_dir: &str, course_id: &str, lesson_file: &str) -> PathBuf {
    let stem = Path::new(lesson_file)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    Path::new(courses_dir)
        .join(course_id)
        .join("chats")
        .join(stem)
}

#[tauri::command]
fn list_lesson_chats(
    app: tauri::AppHandle,
    course_id: String,
    lesson_file: String,
) -> Vec<ChatSummary> {
    let config = load_config(app);
    let dir = chats_dir(&config.courses_dir, &course_id, &lesson_file);
    if !dir.exists() {
        return vec![];
    }
    let mut chats = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(chat) = serde_json::from_str::<Chat>(&content) {
                        chats.push(ChatSummary {
                            id: chat.id,
                            title: chat.title,
                            created_at: chat.created_at,
                            message_count: chat.messages.len(),
                        });
                    }
                }
            }
        }
    }
    chats.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    chats
}

#[tauri::command]
fn load_chat(
    app: tauri::AppHandle,
    course_id: String,
    lesson_file: String,
    chat_id: String,
) -> Result<Chat, String> {
    let config = load_config(app);
    let dir = chats_dir(&config.courses_dir, &course_id, &lesson_file);
    let path = dir.join(format!("{}.json", chat_id));
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_chat(
    app: tauri::AppHandle,
    course_id: String,
    lesson_file: String,
    chat: Chat,
) -> Result<(), String> {
    let config = load_config(app);
    let dir = chats_dir(&config.courses_dir, &course_id, &lesson_file);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.json", chat.id));
    let content = serde_json::to_string_pretty(&chat).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            config_exists,
            load_config,
            save_config,
            get_default_courses_dir,
            list_courses,
            get_course_detail,
            get_lesson_content,
            complete_lesson,
            list_lesson_chats,
            load_chat,
            save_chat,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
