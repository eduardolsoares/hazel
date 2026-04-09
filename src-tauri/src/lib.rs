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
    file_path: Option<String>,
) -> Result<SaveResponse, String> {
    let path = if let Some(path) = file_path {
        path
    } else {
        match app
            .dialog()
            .file()
            .set_file_name("untitled.md".to_string())
            .add_filter("Markdown", &["md"])
            .blocking_save_file()
        {
            Some(file_path) => file_path.to_string(),
            None => {
                return Ok(SaveResponse {
                    success: false,
                    file_path: None,
                    error: Some("No file selected".to_string()),
                })
            }
        }
    };

    let pb = PathBuf::from(&path);
    match std::fs::write(&pb, content.as_bytes()) {
        Ok(_) => Ok(SaveResponse {
            success: true,
            file_path: Some(path),
            error: None,
        }),
        Err(e) => Ok(SaveResponse {
            success: false,
            file_path: None,
            error: Some(e.to_string()),
        }),
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
