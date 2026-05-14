use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["__TAURI__", "core"], js_name = invoke)]
    fn invoke(cmd: &str, args: JsValue) -> js_sys::Promise;
}

pub fn save_markdown_invoke(
    content: String,
    file_path: Option<String>,
    default_name: Option<String>,
) -> js_sys::Promise {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "content": content,
        "filePath": file_path,
        "defaultName": default_name
    }))
    .unwrap();

    invoke("save_markdown", args.into())
}

pub fn check_xelatex_invoke() -> js_sys::Promise {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).unwrap();
    invoke("check_xelatex", args.into())
}

pub fn export_pdf_invoke(content: String, default_name: Option<String>) -> js_sys::Promise {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "content": content,
        "defaultName": default_name
    }))
    .unwrap();

    invoke("export_pdf", args.into())
}

pub fn save_app_state_invoke(state: String) -> js_sys::Promise {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "state": state
    }))
    .unwrap();

    invoke("save_app_state", args.into())
}

pub fn load_app_state_invoke() -> js_sys::Promise {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).unwrap();
    invoke("load_app_state", args.into())
}
