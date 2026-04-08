use tauri_plugin_dialog::DialogExt;

#[tauri::command]
async fn save_file(
    app: tauri::AppHandle,
    file_path: Option<String>
) -> Result<Option<String>, String> {
    let path = if let Some(path) = file_path {
        path
    } else {
        let picked_path = app.dialog()
            .file()
            .add_filter("Markdown", &["md"])
            .set_title("Save File")
            .blocking_save_file();

        match picked_path {
            Some(path_buf) => path_buf.to_string(),
            None => return Ok(None),
        }
    };
    Ok(Some(path))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![save_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
