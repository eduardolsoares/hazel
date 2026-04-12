use serde::{Deserialize, Serialize};
use tauri_plugin_dialog::DialogExt;
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveResponse {
    pub success: bool,
    pub file_path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct XelatexCheckResult {
    pub available: bool,
    pub version: Option<String>,
}

#[allow(dead_code)]
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
    let path = if let Some(path) = filePath {
        path
    } else {
        let name = defaultName.unwrap_or_else(|| "Sem título".to_string());
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

    let pb = PathBuf::from(&path);
    match std::fs::write(&pb, &content) {
        Ok(_) => {
            Ok(SaveResponse {
                success: true,
                file_path: Some(path),
                error: None,
            })
        }
        Err(e) => {
            Ok(SaveResponse {
                success: false,
                file_path: None,
                error: Some(e.to_string()),
            })
        }
    }
}



#[tauri::command]
async fn check_xelatex() -> XelatexCheckResult {
    match std::process::Command::new("xelatex")
        .arg("--version")
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .map(|s| s.to_string());
                return XelatexCheckResult {
                    available: true,
                    version,
                };
            } else {
            }
        }
        Err(_) => {
        }
    }

    XelatexCheckResult {
        available: false,
        version: None,
    }
}

#[tauri::command]
async fn export_pdf(
    app: tauri::AppHandle,
    content: String,
    #[allow(non_snake_case)]
    defaultName: Option<String>,
) -> Result<SaveResponse, String> {

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


    let temp_dir = std::env::temp_dir();
    let temp_md_path = temp_dir.join(format!("hazel_temp_{}.md", std::process::id()));
    if let Err(e) = fs::write(&temp_md_path, &content) {
        return Ok(SaveResponse {
            success: false,
            file_path: None,
            error: Some(format!("Failed to create temp file: {}", e)),
        });
    }

    // Get template path - try dev path first (project root), then production path (next to exe)
    let dev_path = std::path::PathBuf::from("src-tauri/resources/abntex2.latex");
    let exe_path = std::env::current_exe().unwrap_or_default();
    let prod_path = exe_path
        .parent()
        .map(|p| p.join("resources").join("abntex2.latex"))
        .unwrap_or_default();

    let template_path = if dev_path.exists() {
        dev_path
    } else if prod_path.exists() {
        prod_path
    } else {
        dev_path
    };


    // Try system pandoc

    let template_arg = format!("--template={}", template_path.display());
    let input_arg = temp_md_path.to_string_lossy().to_string();

    let cmd_args: Vec<&str> = vec![
        "-V", "documentclass=abntex2",
        &template_arg,
        "-V", "lang=brazil",
        "-V", "papersize=a4paper",
        "-V", "fontsize=12pt",
        "-V", "classoption=twoside",
        "-V", "classoption=openright",
        "-V", "linkcolor=blue",
        &input_arg,
        "-o",
        &pdf_path,
    ];


    let output = std::process::Command::new("pandoc")
        .args(&cmd_args)
        .output();

    // Clean up temp file
    let _ = fs::remove_file(&temp_md_path);

    match output {
        Ok(result) => {
            if result.status.success() {
                Ok(SaveResponse {
                    success: true,
                    file_path: Some(pdf_path),
                    error: None,
                })
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                Ok(SaveResponse {
                    success: false,
                    file_path: None,
                    error: Some(format!("Pandoc error: {}", stderr)),
                })
            }
        }
        Err(e) => {
            Ok(SaveResponse {
                success: false,
                file_path: None,
                error: Some(format!("Failed to run pandoc: {}. Make sure pandoc is installed.", e)),
            })
        }
    }
}

#[tauri::command]
async fn save_app_state(app: tauri::AppHandle, state: String) -> Result<serde_json::Value, String> {

    use tauri_plugin_store::StoreExt;
    let store = match app.store("app_state.json") {
        Ok(s) => s,
        Err(e) => {
            return Err(format!("Failed to get store: {}", e));
        }
    };

    store.set("editor_state", serde_json::Value::String(state.clone()));

    if let Err(e) = store.save() {
        return Err(format!("Failed to save store: {}", e));
    }

    Ok(serde_json::json!({ "success": true, "state": state }))
}

#[tauri::command]
async fn load_app_state(app: tauri::AppHandle) -> Result<serde_json::Value, String> {

    use tauri_plugin_store::StoreExt;
    let store = app.store("app_state.json").map_err(|e| e.to_string())?;

    match store.get("editor_state") {
        Some(value) => {
            let state_str = serde_json::to_string(&value).map_err(|e| e.to_string())?;
            Ok(serde_json::json!({ "success": true, "state": state_str }))
        }
        None => {
            Ok(serde_json::json!({ "success": false }))
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![save_markdown, check_xelatex, export_pdf, save_app_state, load_app_state])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
