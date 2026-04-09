use serde::{Deserialize, Serialize};
use tauri_plugin_dialog::DialogExt;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveResponse {
    pub success: bool,
    pub file_path: Option<String>,
    pub error: Option<String>,
}

#[tauri::command]
async fn save_markdown(
    app: tauri::AppHandle,
    content: String,
) -> Result<bool, String> {
    let path = app
        .dialog()
        .file()
        .set_file_name("untitled".to_string())
        .add_filter("Markdown", &["md"])
        .blocking_save_file();

    match path {
        Some(file_path) => {
            let pb = PathBuf::from(file_path.to_string());
            std::fs::write(&pb, content.as_bytes()).map_err(|e| e.to_string())?;
            Ok(true)
        }
        None => Ok(false),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![save_markdown])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
