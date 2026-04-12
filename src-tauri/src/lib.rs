use serde::{Deserialize, Serialize};
use tauri_plugin_dialog::DialogExt;
use std::path::PathBuf;
use log::info;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveResponse {
    pub success: bool,
    pub file_path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PandocCheckResult {
    pub available: bool,
    pub version: Option<String>,
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

#[tauri::command]
async fn check_pandoc() -> PandocCheckResult {
    info!("Checking pandoc availability...");
    
    match std::process::Command::new("pandoc")
        .arg("--version")
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .map(|s| s.to_string());
                info!("Pandoc available: {:?}", version);
                PandocCheckResult {
                    available: true,
                    version,
                }
            } else {
                info!("Pandoc command failed");
                PandocCheckResult {
                    available: false,
                    version: None,
                }
            }
        }
        Err(e) => {
            info!("Pandoc not found: {}", e);
            PandocCheckResult {
                available: false,
                version: None,
            }
        }
    }
}

#[tauri::command]
async fn export_pdf(
    app: tauri::AppHandle,
    content: String,
    #[allow(non_snake_case)]
    defaultName: Option<String>,
) -> Result<SaveResponse, String> {
    info!("Export PDF requested, content length: {}", content.len());
    
    let name = defaultName.unwrap_or_else(|| "documento".to_string());
    let pdf_path = match app
        .dialog()
        .file()
        .set_file_name(format!("{}.pdf", name))
        .add_filter("PDF", &["pdf"])
        .blocking_save_file()
    {
        Some(path) => path.to_string(),
        None => {
            return Ok(SaveResponse {
                success: false,
                file_path: None,
                error: Some("No file selected".to_string()),
            });
        }
    };

    info!("PDF will be saved to: {}", pdf_path);

    let temp_dir = std::env::temp_dir();
    let temp_md_path = temp_dir.join(format!("hazel_temp_{}.md", std::process::id()));
    
    if let Err(e) = fs::write(&temp_md_path, &content) {
        return Ok(SaveResponse {
            success: false,
            file_path: None,
            error: Some(format!("Failed to create temp file: {}", e)),
        });
    }

    let output = std::process::Command::new("pandoc")
        .arg(temp_md_path.to_string_lossy().to_string())
        .arg("-o")
        .arg(&pdf_path)
        .arg("--template=abntex2")
        .arg("-V")
        .arg("lang=pt-BR")
        .output();

    let _ = fs::remove_file(&temp_md_path);

    match output {
        Ok(result) => {
            if result.status.success() {
                info!("PDF exported successfully to: {}", pdf_path);
                Ok(SaveResponse {
                    success: true,
                    file_path: Some(pdf_path),
                    error: None,
                })
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                info!("Pandoc error: {}", stderr);
                Ok(SaveResponse {
                    success: false,
                    file_path: None,
                    error: Some(format!("Pandoc error: {}", stderr)),
                })
            }
        }
        Err(e) => {
            info!("Failed to run pandoc: {}", e);
            Ok(SaveResponse {
                success: false,
                file_path: None,
                error: Some(format!("Failed to run pandoc: {}", e)),
            })
        }
    }
}

#[tauri::command]
async fn save_app_state(app: tauri::AppHandle, state: String) -> Result<serde_json::Value, String> {
    info!("Saving app state, length: {}", state.len());
    
    use tauri_plugin_store::StoreExt;
    let store = app.store("app_state.json").map_err(|e| e.to_string())?;
    store.set("editor_state", serde_json::Value::String(state.clone()));
    store.save().map_err(|e| e.to_string())?;
    
    info!("App state saved successfully");
    Ok(serde_json::json!({ "success": true, "state": state }))
}

#[tauri::command]
async fn load_app_state(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    info!("Loading app state...");
    
    use tauri_plugin_store::StoreExt;
    let store = app.store("app_state.json").map_err(|e| e.to_string())?;
    
    match store.get("editor_state") {
        Some(value) => {
            let state_str = serde_json::to_string(&value).map_err(|e| e.to_string())?;
            info!("App state loaded, length: {}", state_str.len());
            Ok(serde_json::json!({ "success": true, "state": state_str }))
        }
        None => {
            info!("No saved app state found");
            Ok(serde_json::json!({ "success": false }))
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![save_markdown, check_pandoc, export_pdf, save_app_state, load_app_state])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}