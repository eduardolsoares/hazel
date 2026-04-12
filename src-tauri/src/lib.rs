use serde::{Deserialize, Serialize};
use tauri_plugin_dialog::DialogExt;
use std::path::PathBuf;
use log::info;

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
    #[allow(non_snake_case)]
    filePath: Option<String>,
    #[allow(non_snake_case)]
    defaultName: Option<String>,
) -> Result<SaveResponse, String> {
    info!("Backend received - content length: {}, filePath: {:?}", content.len(), filePath);
    
    let path = if let Some(path) = filePath {
        path
    } else {
        let name = defaultName.unwrap_or_else(|| "untitled".to_string());
        match app
            .dialog()
            .file()
            .set_file_name(name)
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

    info!("Writing to path: {}", path);
    let pb = PathBuf::from(&path);
    match std::fs::write(&pb, &content) {
        Ok(_) => {
            info!("File saved successfully!");
            Ok(SaveResponse {
                success: true,
                file_path: Some(path),
                error: None,
            })
        }
        Err(e) => {
            info!("Error saving file: {}", e);
            Ok(SaveResponse {
                success: false,
                file_path: None,
                error: Some(e.to_string()),
            })
        }
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
