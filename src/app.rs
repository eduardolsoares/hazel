use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yewdux::prelude::*;
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["__TAURI__", "core"], js_name = invoke)]
    fn invoke(cmd: &str, args: JsValue) -> js_sys::Promise;
}

fn save_markdown_invoke(content: String, file_path: Option<String>, default_name: Option<String>) -> js_sys::Promise {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "content": content,
        "filePath": file_path,
        "defaultName": default_name
    }))
    .unwrap();

    invoke("save_markdown", args.into())
}

fn check_xelatex_invoke() -> js_sys::Promise {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).unwrap();
    let promise = invoke("check_xelatex", args.into());
    promise
}

fn export_pdf_invoke(content: String, default_name: Option<String>) -> js_sys::Promise {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "content": content,
        "defaultName": default_name
    }))
    .unwrap();
    invoke("export_pdf", args.into())
}

fn save_app_state_invoke(state: String) -> js_sys::Promise {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "state": state
    }))
    .unwrap();
    invoke("save_app_state", args.into())
}

fn load_app_state_invoke() -> js_sys::Promise {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).unwrap();
    invoke("load_app_state", args.into())
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub id: usize,
    pub block_type: BlockType,
    pub content: String,
    pub prev: Option<usize>,
    pub next: Option<usize>,
}

impl Block {
    pub fn new(id: usize, block_type: BlockType) -> Self {
        Self {
            id,
            block_type,
            content: String::new(),
            prev: None,
            next: None,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum BlockType {
    Paragraph,
    Heading1,
    Heading2,
    Heading3,
    Image,
    Citation,
    CodeBlock,
    BulletList,
    NumberedList,
    Quote,
    HorizontalRule,
    // Estrutura do texto (ABNT)
    Introducao,
    Desenvolvimento,
    Conclusao,
    // Ambientes abntex2
    Teorema,
    Prova,
    Definicao,
    Exemplo,
    Observacao,
    CitacaoLonga,
}

impl Default for Block {
    fn default() -> Self {
        Block::new(0, BlockType::Paragraph)
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Buffer {
    pub head: Option<usize>,
    pub tail: Option<usize>,
    pub blocks: std::collections::HashMap<usize, Block>,
    pub length: usize,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            blocks: std::collections::HashMap::new(),
            length: 0,
        }
    }

    pub fn from_blocks(blocks: Vec<Block>) -> Self {
        let mut buffer = Self::new();
        let mut prev_id: Option<usize> = None;

        for block in blocks {
            let id = block.id;
            if let Some(prev) = prev_id {
                if let Some(b) = buffer.blocks.get_mut(&prev) {
                    b.next = Some(id);
                }
            } else {
                buffer.head = Some(id);
            }
            buffer.blocks.insert(
                id,
                Block {
                    prev: prev_id,
                    ..block
                },
            );
            prev_id = Some(id);
            buffer.length += 1;
        }

        if let Some(last_id) = prev_id {
            buffer.tail = Some(last_id);
        }

        buffer
    }

    pub fn push_back(&mut self, block: Block) {
        let id = block.id;

        if let Some(tail_id) = self.tail {
            if let Some(b) = self.blocks.get_mut(&tail_id) {
                b.next = Some(id);
            }
        } else {
            self.head = Some(id);
        }

        self.blocks.insert(
            id,
            Block {
                prev: self.tail,
                ..block
            },
        );
        self.tail = Some(id);
        self.length += 1;
    }

    pub fn to_vec(&self) -> Vec<Block> {
        let mut result = Vec::new();
        let mut current = self.head;

        while let Some(id) = current {
            if let Some(block) = self.blocks.get(&id) {
                result.push(block.clone());
            }
            current = self.blocks.get(&id).and_then(|b| b.next);
        }

        result
    }

    pub fn to_markdown(&self) -> String {
        let mut markdown = String::new();
        let mut current = self.head;

        while let Some(id) = current {
            if let Some(block) = self.blocks.get(&id) {
                match block.block_type {
                    BlockType::Heading1 => {
                        markdown.push_str(&format!("# {}\n\n", block.content));
                    }
                    BlockType::Heading2 => {
                        markdown.push_str(&format!("## {}\n\n", block.content));
                    }
                    BlockType::Heading3 => {
                        markdown.push_str(&format!("### {}\n\n", block.content));
                    }
                    BlockType::Paragraph => {
                        if !block.content.is_empty() {
                            markdown.push_str(&format!("{}\n\n", block.content));
                        }
                    }
                    BlockType::Image => {
                        markdown.push_str(&format!("![{}]({})\n\n", block.content, block.content));
                    }
                    BlockType::Citation => {
                        if !block.content.is_empty() {
                            markdown.push_str(&format!("> {}\n\n", block.content));
                        }
                    }
                    BlockType::CodeBlock => {
                        if !block.content.is_empty() {
                            markdown.push_str(&format!("```\n{}\n```\n\n", block.content));
                        }
                    }
                    BlockType::BulletList => {
                        markdown.push_str(&format!("- {}\n", block.content));
                    }
                    BlockType::NumberedList => {
                        markdown.push_str(&format!("1. {}\n", block.content));
                    }
                    BlockType::Quote => {
                        markdown.push_str(&format!("> {}\n\n", block.content));
                    }
                    BlockType::HorizontalRule => {
                        markdown.push_str("---\n\n");
                    }
                    BlockType::Introducao => {
                        markdown.push_str(&format!("# Introdução\n\n{}\n\n", block.content));
                    }
                    BlockType::Desenvolvimento => {
                        markdown.push_str(&format!("# Desenvolvimento\n\n{}\n\n", block.content));
                    }
                    BlockType::Conclusao => {
                        markdown.push_str(&format!("# Conclusão\n\n{}\n\n", block.content));
                    }
                    BlockType::Teorema => {
                        if !block.content.is_empty() {
                            markdown.push_str(&format!("::: .theorem\n{}\n:::\n\n", block.content));
                        }
                    }
                    BlockType::Prova => {
                        if !block.content.is_empty() {
                            markdown.push_str(&format!("::: .proof\n{}\n:::\n\n", block.content));
                        }
                    }
                    BlockType::Definicao => {
                        if !block.content.is_empty() {
                            markdown.push_str(&format!("::: .definition\n{}\n:::\n\n", block.content));
                        }
                    }
                    BlockType::Exemplo => {
                        if !block.content.is_empty() {
                            markdown.push_str(&format!("::: .example\n{}\n:::\n\n", block.content));
                        }
                    }
                    BlockType::Observacao => {
                        if !block.content.is_empty() {
                            markdown.push_str(&format!("::: .observation\n{}\n:::\n\n", block.content));
                        }
                    }
                    BlockType::CitacaoLonga => {
                        if !block.content.is_empty() {
                            markdown.push_str(&format!("::: citacao\n{}\n:::\n\n", block.content));
                        }
                    }
                }
            }
            current = self.blocks.get(&id).and_then(|b| b.next);
        }
        markdown
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Tab {
    pub id: usize,
    pub name: String,
    pub title: String,
    pub buffer: Buffer,
    pub file_path: Option<String>,
    pub is_dirty: bool,
    pub block_order: Vec<usize>,
    pub saved_content: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct SlashOption {
    pub block_type: Option<BlockType>,
    pub label: String,
    pub icon: &'static str,
    pub category: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct SlashCategory {
    pub name: String,
    pub options: Vec<SlashOption>,
}


#[derive(Clone, PartialEq, Store)]
pub struct EditorState {
    pub tabs: Vec<Tab>,
    pub active_tab_id: usize,
    pub next_tab_id: usize,
    pub next_block_id: usize,
    pub show_slash_menu: bool,
    pub slash_menu_block_id: Option<usize>,
    pub focused_block_id: Option<usize>,
    pub show_save_modal: bool,
    pub save_modal_filename: String,
    pub xelatex_available: bool,
    pub xelatex_version: Option<String>,
    pub save_modal_export_type: ExportType,
    pub show_settings_modal: bool,
    pub dark_mode: bool,
    pub notification: Option<Notification>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    pub message: String,
    pub is_error: bool,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorStateDto {
    pub tabs: Vec<Tab>,
    pub active_tab_id: usize,
    pub next_tab_id: usize,
    pub next_block_id: usize,
    pub show_settings_modal: bool,
    pub dark_mode: bool,
}

impl From<&EditorState> for EditorStateDto {
    fn from(state: &EditorState) -> Self {
        Self {
            tabs: state.tabs.clone(),
            active_tab_id: state.active_tab_id,
            next_tab_id: state.next_tab_id,
            next_block_id: state.next_block_id,
            show_settings_modal: false,
            dark_mode: false,
        }
    }
}

impl From<EditorStateDto> for EditorState {
    fn from(dto: EditorStateDto) -> Self {
        let mut state = Self::default();
        state.tabs = dto.tabs;
        state.active_tab_id = dto.active_tab_id;
        state.next_tab_id = dto.next_tab_id;
        state.next_block_id = dto.next_block_id;
        state.show_settings_modal = dto.show_settings_modal;
        state.dark_mode = dto.dark_mode;
        state
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ExportType {
    Markdown,
    Pdf,
}

impl Default for EditorState {
    fn default() -> Self {
        let mut buffer = Buffer::new();
        buffer.push_back(Block::new(0, BlockType::Paragraph));

        Self {
            tabs: vec![Tab {
                id: 0,
                name: "Sem título.md".to_string(),
                title: "Sem título".to_string(),
                buffer,
                file_path: None,
                is_dirty: false,
                block_order: vec![0],
                saved_content: None,
            }],
            active_tab_id: 0,
            next_tab_id: 1,
            next_block_id: 1,
            show_slash_menu: false,
            slash_menu_block_id: None,
            focused_block_id: None,
            show_save_modal: false,
            save_modal_filename: "Sem título".to_string(),
            xelatex_available: false,
            xelatex_version: None,
            save_modal_export_type: ExportType::Markdown,
            show_settings_modal: false,
            dark_mode: false,
            notification: None,
        }
    }
}

fn get_slash_categories() -> Vec<SlashCategory> {
    vec![
        SlashCategory {
            name: "Básico".to_string(),
            options: vec![
                SlashOption {
                    block_type: Some(BlockType::Paragraph),
                    label: "Parágrafo".to_string(),
                    icon: "¶",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Heading1),
                    label: "Título 1".to_string(),
                    icon: "H1",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Heading2),
                    label: "Título 2".to_string(),
                    icon: "H2",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Heading3),
                    label: "Título 3".to_string(),
                    icon: "H3",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::BulletList),
                    label: "Lista com marcadores".to_string(),
                    icon: "•",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::NumberedList),
                    label: "Lista numerada".to_string(),
                    icon: "1.",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Quote),
                    label: "Citação".to_string(),
                    icon: "❝",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::CodeBlock),
                    label: "Código".to_string(),
                    icon: "</>",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Image),
                    label: "Imagem".to_string(),
                    icon: "🖼",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::HorizontalRule),
                    label: "Linha horizontal".to_string(),
                    icon: "—",
                    category: None,
                },
            ],
        },
        SlashCategory {
            name: "Estrutura do Texto".to_string(),
            options: vec![
                SlashOption {
                    block_type: Some(BlockType::Introducao),
                    label: "Introdução".to_string(),
                    icon: "#",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Desenvolvimento),
                    label: "Desenvolvimento".to_string(),
                    icon: "=",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Conclusao),
                    label: "Conclusão".to_string(),
                    icon: "✓",
                    category: None,
                },
            ],
        },
        SlashCategory {
            name: "Ambientes".to_string(),
            options: vec![
                SlashOption {
                    block_type: Some(BlockType::Teorema),
                    label: "Teorema".to_string(),
                    icon: "▢",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Prova),
                    label: "Prova".to_string(),
                    icon: "∎",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Definicao),
                    label: "Definição".to_string(),
                    icon: "≡",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Exemplo),
                    label: "Exemplo".to_string(),
                    icon: "ex",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Observacao),
                    label: "Observação".to_string(),
                    icon: "i",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::CitacaoLonga),
                    label: "Citação longa".to_string(),
                    icon: "❞",
                    category: None,
                },
            ],
        },
    ]
}

#[function_component(App)]
pub fn app() -> Html {
    let (state, dispatch) = use_store::<EditorState>();

    let dismiss_notification = {
        let dispatch = dispatch.clone();
        Callback::from(move |_| {
            dispatch.reduce_mut(move |state| {
                state.notification = None;
            });
        })
    };

    let open_save_modal = {
        let dispatch = dispatch.clone();
        Callback::from(move |_: ()| {
            let state = dispatch.get();
            if let Some(tab) = state.tabs.iter().find(|t| t.id == state.active_tab_id) {
                dispatch.reduce_mut(move |state| {
                    state.show_save_modal = true;
                    state.save_modal_filename = tab.title.clone();
                    state.save_modal_export_type = ExportType::Markdown;
                });
            }
        })
    };

    let close_save_modal = {
        let dispatch = dispatch.clone();
        Callback::from(move |_| {
            dispatch.reduce_mut(move |state| {
                state.show_save_modal = false;
            });
        })
    };

    let save_callback = {
        let dispatch = dispatch.clone();
        Callback::from(move |_: ()| {
            web_sys::console::log_1(&"Ctrl+S pressed - starting save".into());
            let state = dispatch.get();
            web_sys::console::log_1(&format!("Current state has {} tabs", state.tabs.len()).into());
            let dto = EditorStateDto::from(&*state);
            if let Ok(state_json) = serde_json::to_string(&dto) {
                web_sys::console::log_1(
                    &format!("Serialized state, length: {}", state_json.len()).into(),
                );
                let dispatch2 = dispatch.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    web_sys::console::log_1(&"Awaiting save...".into());
                    let promise = save_app_state_invoke(state_json);
                    let result = wasm_bindgen_futures::JsFuture::from(promise).await;

                    match result {
                        Ok(value) => {
                            web_sys::console::log_1(&format!("Save result: {:?}", value).into());
                            if let Ok(result_obj) = serde_wasm_bindgen::from_value::<serde_json::Value>(value) {
                                let success = result_obj
                                    .get("success")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false);
                                web_sys::console::log_1(&format!("Save success: {}", success).into());
                                if success {
                                    dispatch2.reduce_mut(move |state| {
                                        for tab in state.tabs.iter_mut() {
                                            let content = tab.buffer.to_markdown();
                                            web_sys::console::log_1(&format!("Setting saved_content, len={}", content.len()).into());
                                            tab.saved_content = Some(content);
                                            tab.is_dirty = false;
                                        }
                                    });
                                    web_sys::console::log_1(&"State saved, is_dirty cleared".into());
                                }
                            }
                        }
                        Err(e) => {
                            web_sys::console::log_1(&format!("Save error: {:?}", e).into());
                        }
                    }
                });
            }
        })
    };

    let handle_save = {
        let dispatch = dispatch.clone();
        Callback::from(move |_| {
            let state = dispatch.get();
            if let Some(tab) = state.tabs.iter().find(|t| t.id == state.active_tab_id) {
                let content = tab.buffer.to_markdown();
                let filename = state.save_modal_filename.clone();

                if state.save_modal_export_type == ExportType::Markdown {
                    let file_path = tab.file_path.clone();
                    let default_name = Some(format!("{}.md", filename.replace(' ', "_")));
                    let dispatch_for_notify = dispatch.clone();
                    let active_tab_id = state.active_tab_id;

                    wasm_bindgen_futures::spawn_local(async move {
                        let promise = save_markdown_invoke(content, file_path, default_name);
                        let result = wasm_bindgen_futures::JsFuture::from(promise).await;

                        match result {
                            Ok(value) => {
                                if let Ok(result_obj) = serde_wasm_bindgen::from_value::<serde_json::Value>(value) {
                                    let success = result_obj.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                                    let notification = if success {
                                        let path = result_obj.get("file_path").and_then(|v| v.as_str())
                                            .map(|s| format!("Arquivo salvo: {}", s))
                                            .unwrap_or_else(|| "Salvo com sucesso".to_string());
                                        Notification { message: path, is_error: false }
                                    } else {
                                        let error = result_obj.get("error").and_then(|v| v.as_str())
                                            .unwrap_or("Erro ao salvar")
                                            .to_string();
                                        Notification { message: error, is_error: true }
                                    };
                                    dispatch_for_notify.reduce_mut(move |state| {
                                        state.notification = Some(notification);
                                        if let Some(t) = state.tabs.iter_mut().find(|t| t.id == active_tab_id)
                                        {
                                            t.is_dirty = false;
                                        }
                                        state.show_save_modal = false;
                                    });
                                }
                            },
                            Err(_) => {}
                        }
                    });

                    // Close modal immediately, notification will show async
                } else if state.xelatex_available {
                    let default_name = Some(filename.replace(' ', "_"));

                    // Use async/await pattern
                    let content_clone = content.clone();
                    let default_name_clone = default_name.clone();
                    let dispatch_for_notify = dispatch.clone();

                    wasm_bindgen_futures::spawn_local(async move {
                        web_sys::console::log_1(&"Starting PDF export async".into());
                        let promise = export_pdf_invoke(content_clone, default_name_clone);
                        let result = wasm_bindgen_futures::JsFuture::from(promise).await;

                        web_sys::console::log_1(&"PDF export async got result".into());

                        match result {
                            Ok(value) => {
                                if let Ok(result_obj) = serde_wasm_bindgen::from_value::<serde_json::Value>(value) {
                                    let success = result_obj.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                                    web_sys::console::log_1(&format!("Export success: {}", success).into());

                                    let notification = if success {
                                        let path = result_obj.get("file_path").and_then(|v| v.as_str())
                                            .map(|s| format!("PDF salvo em: {}", s))
                                            .unwrap_or_else(|| "PDF salvo com sucesso".to_string());
                                        Notification { message: path, is_error: false }
                                    } else {
                                        let error = result_obj.get("error").and_then(|v| v.as_str())
                                            .unwrap_or("Erro ao exportar PDF")
                                            .to_string();
                                        Notification { message: error, is_error: true }
                                    };
                                    dispatch_for_notify.reduce_mut(move |state| {
                                        state.notification = Some(notification);
                                    });
                                }
                            },
                            Err(e) => {
                                web_sys::console::log_1(&format!("PDF export error: {:?}", e).into());
                            }
                        }
                    });

                    dispatch.reduce_mut(move |state| {
                        if let Some(t) = state.tabs.iter_mut().find(|t| t.id == state.active_tab_id)
                        {
                            t.is_dirty = false;
                        }
                        state.show_save_modal = false;
                    });
                } else {
                    dispatch.reduce_mut(move |state| {
                        state.show_save_modal = false;
                    });
                }
            }
        })
    };

    let update_modal_filename = {
        let dispatch = dispatch.clone();
        Callback::from(move |filename: String| {
            dispatch.reduce_mut(move |state| {
                state.save_modal_filename = filename;
            });
        })
    };

    let set_export_type = {
        let dispatch = dispatch.clone();
        Callback::from(move |export_type: ExportType| {
            dispatch.reduce_mut(move |state| {
                state.save_modal_export_type = export_type;
            });
        })
    };

    let dispatch_for_load = dispatch.clone();

    use_effect(move || {
        let dispatch = dispatch_for_load.clone();
        let promise = load_app_state_invoke();
        let _ = promise.then(&wasm_bindgen::closure::Closure::wrap(Box::new(
            move |result: JsValue| {
                web_sys::console::log_1(&format!("Load app state result: {:?}", result).into());
                if let Ok(result_obj) = serde_wasm_bindgen::from_value::<serde_json::Value>(result)
                {
                    if let Some(state_str) = result_obj
                        .get("success")
                        .and_then(|v| v.as_bool())
                        .and_then(|success| {
                            if success {
                                result_obj
                                    .get("state")
                                    .and_then(|v| v.as_str())
                                    .map(String::from)
                            } else {
                                None
                            }
                        })
                    {
                        if let Ok(loaded_dto) = serde_json::from_str::<EditorStateDto>(&state_str) {
                            web_sys::console::log_1(&"State loaded successfully from store".into());
                            let loaded_state: EditorState = loaded_dto.into();
                            dispatch.set(loaded_state);
                        } else {
                            web_sys::console::log_1(&"Failed to parse loaded state".into());
                        }
                    } else {
                        web_sys::console::log_1(&"No saved state found, using default".into());
                    }
                }
            },
        )
            as Box<dyn FnMut(JsValue)>));
        || {}
    });

    {
        use std::sync::OnceLock;
        static CHECKED: OnceLock<()> = OnceLock::new();

        let dispatch = dispatch.clone();
        if CHECKED.get().is_none() {
            let _ = CHECKED.set(());

            wasm_bindgen_futures::spawn_local(async move {
                let promise = check_xelatex_invoke();
                let result = match wasm_bindgen_futures::JsFuture::from(promise).await {
                    Ok(value) => value,
                    Err(_) => return,
                };

                if let Ok(result_obj) = serde_wasm_bindgen::from_value::<serde_json::Value>(result) {
                    if let Some(available) = result_obj.get("available").and_then(|v| v.as_bool()) {
                        let version = result_obj
                            .get("version")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        dispatch.reduce_mut(move |state| {
                            state.xelatex_available = available;
                            state.xelatex_version = version;
                        });
                    }
                }
            });
        }
    }

    let dispatch_for_keys = dispatch.clone();

    use_effect(move || {
        use gloo_events::EventListener;
        use std::sync::OnceLock;

        static REGISTERED: OnceLock<()> = OnceLock::new();

        if REGISTERED.get().is_none() {
            let save = save_callback.clone();
            let dispatch_for_escape = dispatch_for_keys.clone();
            let _ = REGISTERED.set(());

            let listener = EventListener::new(
                &web_sys::window().unwrap().unchecked_ref(),
                "keydown",
                move |event| {
                    let e = event.unchecked_ref::<web_sys::KeyboardEvent>();
                    if e.ctrl_key() && e.key() == "s" {
                        e.prevent_default();
                        save.emit(());
                    } else if e.key() == "Escape" {
                        e.prevent_default();
                        dispatch_for_escape.reduce_mut(move |state| {
                            state.show_settings_modal = !state.show_settings_modal;
                        });
                    }
                },
            );
            listener.forget();
        }

        || {}
    });

    let switch_tab = {
        let dispatch = dispatch.clone();
        Callback::from(move |tab_id: usize| {
            web_sys::console::log_1(&format!("switch_tab called with: {}", tab_id).into());
            let captured_id = tab_id;
            dispatch.reduce_mut(move |state| {
                web_sys::console::log_1(
                    &format!(
                        "switch_tab - received tab_id: {}, setting active to it",
                        captured_id
                    )
                    .into(),
                );
                state.active_tab_id = captured_id;
                state.show_slash_menu = false;
            });
        })
    };

    let add_tab = {
        let dispatch = dispatch.clone();
        Callback::from(move |_| {
            dispatch.reduce_mut(move |state| {
                let new_id = state.next_tab_id;
                state.next_tab_id += 1;

                let mut buffer = Buffer::new();
                let block_id = state.next_block_id;
                buffer.push_back(Block::new(block_id, BlockType::Paragraph));
                state.next_block_id += 1;

                state.tabs.push(Tab {
                    id: new_id,
                    name: "Sem título.md".to_string(),
                    title: "Sem título".to_string(),
                    buffer,
                    file_path: None,
                    is_dirty: false,
                    block_order: vec![block_id],
                    saved_content: None,
                });
                state.active_tab_id = new_id;
            });
        })
    };

    let close_tab = {
        let dispatch = dispatch.clone();
        Callback::from(move |tab_id: usize| {
            dispatch.reduce_mut(move |state| {
                if state.tabs.len() > 1 {
                    state.tabs.retain(|t| t.id != tab_id);
                    if state.active_tab_id == tab_id {
                        state.active_tab_id = state.tabs[0].id;
                    }
                }
            });
        })
    };

    let hide_slash_menu = {
        let dispatch = dispatch.clone();
        Callback::from(move |_| {
            dispatch.reduce_mut(move |state| {
                state.show_slash_menu = false;
                state.slash_menu_block_id = None;
            });
        })
    };

    let show_slash_menu = {
        let dispatch = dispatch.clone();
        Callback::from(move |block_id: usize| {
            dispatch.reduce_mut(move |state| {
                state.show_slash_menu = true;
                state.slash_menu_block_id = Some(block_id);
            });
        })
    };

    let select_slash_option = {
        let dispatch = dispatch.clone();
        Callback::from(move |block_type: Option<BlockType>| {
            if let Some(bt) = block_type {
                dispatch.reduce_mut(move |state| {
                    if let Some(block_id) = state.slash_menu_block_id {
                        if let Some(tab) = state.tabs.iter_mut().find(|t| t.id == state.active_tab_id) {
                            if let Some(block) = tab.buffer.blocks.get_mut(&block_id) {
                                block.block_type = bt.clone();
                                block.content = String::new();
                            }
                        }
                        state.focused_block_id = state.slash_menu_block_id;
                    }
                    state.show_slash_menu = false;
                    state.slash_menu_block_id = None;
                });
            }
        })
    };

    let handle_enter = {
        let dispatch = dispatch.clone();
        Callback::from(move |block_id: usize| {
            dispatch.reduce_mut(move |state| {
                if let Some(tab) = state.tabs.iter_mut().find(|t| t.id == state.active_tab_id) {
                    if let Some(pos) = tab.block_order.iter().position(|&id| id == block_id) {
                        let new_block_id = state.next_block_id;
                        state.next_block_id += 1;

                        let new_block = Block::new(new_block_id, BlockType::Paragraph);

                        if let Some(current_block) = tab.buffer.blocks.get_mut(&block_id) {
                            let next_id = current_block.next;
                            current_block.next = Some(new_block_id);

                            let mut new_block_with_links = new_block;
                            new_block_with_links.prev = Some(block_id);
                            new_block_with_links.next = next_id;

                            tab.buffer.blocks.insert(new_block_id, new_block_with_links);

                            if let Some(nid) = next_id {
                                if let Some(next_block) = tab.buffer.blocks.get_mut(&nid) {
                                    next_block.prev = Some(new_block_id);
                                }
                            } else {
                                tab.buffer.tail = Some(new_block_id);
                            }
                        }

                        tab.block_order.insert(pos + 1, new_block_id);
                        state.focused_block_id = Some(new_block_id);
                    }
                }
            });
        })
    };

    let handle_up_arrow = {
        let dispatch = dispatch.clone();
        Callback::from(move |block_id: usize| {
            dispatch.reduce_mut(move |state| {
                if let Some(tab) = state.tabs.iter_mut().find(|t| t.id == state.active_tab_id) {
                    if let Some(pos) = tab.block_order.iter().position(|&id| id == block_id) {
                        if pos > 0 {
                            let prev_id = tab.block_order[pos - 1];
                            state.focused_block_id = Some(prev_id);
                        }
                    }
                }
            });
        })
    };

    let handle_down_arrow = {
        let dispatch = dispatch.clone();
        Callback::from(move |block_id: usize| {
            dispatch.reduce_mut(move |state| {
                if let Some(tab) = state.tabs.iter_mut().find(|t| t.id == state.active_tab_id) {
                    if let Some(pos) = tab.block_order.iter().position(|&id| id == block_id) {
                        if pos < tab.block_order.len() - 1 {
                            let next_id = tab.block_order[pos + 1];
                            state.focused_block_id = Some(next_id);
                        }
                    }
                }
            });
        })
    };

    let handle_backspace = {
        let dispatch = dispatch.clone();
        Callback::from(move |block_id: usize| {
            dispatch.reduce_mut(move |state| {
                if let Some(tab) = state.tabs.iter_mut().find(|t| t.id == state.active_tab_id) {
                    if let Some(block) = tab.buffer.blocks.get(&block_id) {
                        if block.content.is_empty() && tab.block_order.len() > 1 {
                            if let Some(pos) = tab.block_order.iter().position(|&id| id == block_id)
                            {
                                tab.block_order.remove(pos);
                                tab.buffer.blocks.remove(&block_id);
                                if pos > 0 {
                                    state.focused_block_id = Some(tab.block_order[pos - 1]);
                                } else if !tab.block_order.is_empty() {
                                    state.focused_block_id = Some(tab.block_order[0]);
                                }
                            }
                        }
                    }
                    let current_content = tab.buffer.to_markdown();
                    let saved = tab.saved_content.clone();
                    let is_same = saved.as_ref().map(|s| s == &current_content).unwrap_or(false);
                    tab.is_dirty = !is_same;
                }
            });
        })
    };

    let handle_delete = {
        let dispatch = dispatch.clone();
        Callback::from(move |block_id: usize| {
            dispatch.reduce_mut(move |state| {
                if let Some(tab) = state.tabs.iter_mut().find(|t| t.id == state.active_tab_id) {
                    if let Some(block) = tab.buffer.blocks.get(&block_id) {
                        if block.content.is_empty() && tab.block_order.len() > 1 {
                            if let Some(pos) = tab.block_order.iter().position(|&id| id == block_id)
                            {
                                tab.block_order.remove(pos);
                                tab.buffer.blocks.remove(&block_id);
                                if pos < tab.block_order.len() {
                                    state.focused_block_id = Some(tab.block_order[pos]);
                                } else if !tab.block_order.is_empty() {
                                    state.focused_block_id =
                                        Some(tab.block_order[tab.block_order.len() - 1]);
                                }
                            }
                        }
                    }
                    let current_content = tab.buffer.to_markdown();
                    let saved = tab.saved_content.clone();
                    let is_same = saved.as_ref().map(|s| s == &current_content).unwrap_or(false);
                    tab.is_dirty = !is_same;
                }
            });
        })
    };

    let active_tab = state
        .tabs
        .iter()
        .find(|t| t.id == state.active_tab_id)
        .cloned();
    web_sys::console::log_1(
        &format!(
            "render - active_tab_id: {}, title: {:?}",
            state.active_tab_id,
            active_tab.as_ref().map(|t| &t.title)
        )
        .into(),
    );

    html! {
        <div class={classes!("app", if state.dark_mode { "dark-mode" } else { "" })}>
            <div class="tab-bar">
                <div class="tabs">
                    {for state.tabs.iter().map(|tab| {
                        let is_active = tab.id == state.active_tab_id;
                        let tab_id = tab.id;
                        let switch = switch_tab.clone();
                        let close = close_tab.clone();
                        html! {
                            <div
                                class={classes!("tab", if is_active { "active" } else { "" })}
                                onclick={move |_| switch.emit(tab_id)}
                            >
                                <span class="tab-name">
                                    {if tab.is_dirty {
                                        html! { <span class="unsaved-indicator">{"●"}</span> }
                                    } else {
                                        html! {}
                                    }}
                                    {&tab.name}
                                </span>
                                <button
                                    class="tab-close"
                                    onclick={move |e: MouseEvent| {
                                        e.stop_propagation();
                                        close.emit(tab_id);
                                    }}
                                >
                                    {"×"}
                                </button>
                            </div>
                        }
                    })}
                </div>
                <button class="new-tab-btn" onclick={add_tab}>
                    {"+"}
                </button>
            </div>

            <div class="editor-container">
                {if let Some(tab) = active_tab.clone() {
                    let tab_id = tab.id;
                    let tab_title = tab.title.clone();
                    let tab_id_for_title = tab.id;
                    let dispatch_for_title = dispatch.clone();
                    let dispatch_for_title_blur = dispatch.clone();
                    html! {
                        <div class="page" key={tab_id}>
                            <div class="page-title" contenteditable="true"
                                oninput={Callback::from(move |e: InputEvent| {
                                    if let Some(target) = e.target_dyn_into::<web_sys::HtmlElement>() {
                                        let text = target.text_content().unwrap_or_default();
                                        let name = if text.is_empty() { "Sem título".to_string() } else { text.clone() };
                                        dispatch_for_title.reduce_mut(move |state| {
                                            if let Some(t) = state.tabs.iter_mut().find(|t| t.id == tab_id_for_title) {
                                                t.title = text;
                                                t.name = format!("{}.md", name.replace(' ', "_"));
                                                let current_content = t.buffer.to_markdown();
                                                let is_same = t.saved_content.as_ref()
                                                    .map(|s| s == &current_content)
                                                    .unwrap_or(false);
                                                t.is_dirty = !is_same;
                                            }
                                        });
                                    }
                                })}
                                onblur={Callback::from(move |e: FocusEvent| {
                                    if let Some(target) = e.target_dyn_into::<web_sys::HtmlElement>() {
                                        let text = target.text_content().unwrap_or_default();
                                        let name = if text.is_empty() { "Sem título".to_string() } else { text.clone() };
                                        dispatch_for_title_blur.reduce_mut(move |state| {
                                            if let Some(t) = state.tabs.iter_mut().find(|t| t.id == tab_id_for_title) {
                                                t.title = text;
                                                t.name = format!("{}.md", name.replace(' ', "_"));
                                                let current_content = t.buffer.to_markdown();
                                                let is_same = t.saved_content.as_ref()
                                                    .map(|s| s == &current_content)
                                                    .unwrap_or(false);
                                                t.is_dirty = !is_same;
                                            }
                                        });
                                    }
                                })}
                            >
                                {&tab_title}
                            </div>

                            <div class="blocks">
                                {for tab.buffer.to_vec().iter().map(|block| {
                                    let is_menu_target = state.show_slash_menu && state.slash_menu_block_id == Some(block.id);
                                    let hide_slash = hide_slash_menu.clone();
                                    let dispatch_clone = dispatch.clone();
                                    let focused_id = state.focused_block_id;
                                    html! {
                                        <>
                                            <BlockComponent
                                                key={block.id}
                                                block={block.clone()}
                                                on_show_slash_menu={show_slash_menu.clone()}
                                                on_keydown={Callback::from(move |key: String| {
                                                    if key == "Backspace" {
                                                        hide_slash.emit(());
                                                    }
                                                })}
                                                on_backspace={handle_backspace.clone()}
                                                on_delete={handle_delete.clone()}
                                                on_change={let dispatch2 = dispatch_clone.clone(); Callback::from(move |(id, content): (usize, String)| {
                                                    dispatch2.reduce_mut(move |state| {
                                                        if let Some(tab) = state.tabs.iter_mut().find(|t| t.id == state.active_tab_id) {
                                                            if let Some(block) = tab.buffer.blocks.get_mut(&id) {
                                                                block.content = content;
                                                            }
                                                            let current_content = tab.buffer.to_markdown();
                                                            let saved = tab.saved_content.clone();
                                                            let is_same = saved.as_ref()
                                                                .map(|s| s == &current_content)
                                                                .unwrap_or(false);
                                                            web_sys::console::log_1(&format!("on_change: current='{}', saved={:?}, is_same={}",
                                                                current_content.len(),
                                                                saved.as_ref().map(|s| s.len()),
                                                                is_same).into());
                                                            tab.is_dirty = !is_same;
                                                        }
                                                    });
                                                })}
                                                on_blur={let dispatch2 = dispatch_clone.clone(); Callback::from(move |(id, content): (usize, String)| {
                                                    dispatch2.reduce_mut(move |state| {
                                                        if let Some(tab) = state.tabs.iter_mut().find(|t| t.id == state.active_tab_id) {
                                                            if let Some(block) = tab.buffer.blocks.get_mut(&id) {
                                                                block.content = content;
                                                            }
                                                            let current_content = tab.buffer.to_markdown();
                                                            let is_same = tab.saved_content.as_ref()
                                                                .map(|s| s == &current_content)
                                                                .unwrap_or(false);
                                                            tab.is_dirty = !is_same;
                                                        }
                                                    });
                                                })}
                                                on_enter={handle_enter.clone()}
                                                on_up_arrow={handle_up_arrow.clone()}
                                                on_down_arrow={handle_down_arrow.clone()}
                                                on_focus={Callback::from(|_| {})}
                                                focused_block_id={focused_id}
                                            />
                                            {if is_menu_target {
                                                html! {
                                                    <SlashMenu
                                                        categories={get_slash_categories()}
                                                        on_select={select_slash_option.clone()}
                                                        on_close={hide_slash_menu.clone()}
                                                    />
                                                }
                                            } else {
                                                html! {}
                                            }}
                                        </>
                                    }
                                })}
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }}
            </div>

            {if state.show_save_modal {
                let _dispatch_for_close = dispatch.clone();
                let _dispatch_for_save = dispatch.clone();
                let dispatch_for_filename = dispatch.clone();
                let dispatch_for_type_md = dispatch.clone();
                let dispatch_for_type_pdf = dispatch.clone();
                let filename = state.save_modal_filename.clone();
                let export_type = state.save_modal_export_type.clone();
                let xelatex_available = state.xelatex_available;
                html! {
                    <div class="modal-overlay">
                        <div class="modal">
                            <div class="modal-header">
                                {"exportar arquivo"}
                                <button class="modal-close" onclick={close_save_modal.clone()}>{"×"}</button>
                            </div>
                            <div class="modal-body">
                                <div class="modal-input-group">
                                    <label>{"Nome do arquivo:"}</label>
                                    <input
                                        type="text"
                                        class="modal-input"
                                        value={filename}
                                        oninput={Callback::from(move |e: InputEvent| {
                                            if let Some(target) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                dispatch_for_filename.reduce_mut(move |state| {
                                                    state.save_modal_filename = target.value();
                                                });
                                            }
                                        })}
                                    />
                                </div>
                                <div class="modal-radio-group">
                                    <label class="modal-radio-label">
                                        <input
                                            type="radio"
                                            name="export_type"
                                            checked={export_type == ExportType::Markdown}
                                            onchange={Callback::from(move |_| {
                                                dispatch_for_type_md.reduce_mut(move |state| {
                                                    state.save_modal_export_type = ExportType::Markdown;
                                                });
                                            })}
                                        />
                                        <span>{"Markdown (.md)"}</span>
                                    </label>
                                    <label class={classes!("modal-radio-label", if !xelatex_available { "disabled" } else { "" })}>
                                        <input
                                            type="radio"
                                            name="export_type"
                                            checked={export_type == ExportType::Pdf}
                                            disabled={!xelatex_available}
                                            onchange={Callback::from(move |_| {
                                                dispatch_for_type_pdf.reduce_mut(move |state| {
                                                    state.save_modal_export_type = ExportType::Pdf;
                                                });
                                            })}
                                        />
                                        <span>{"PDF (ABNT)"}</span>
                                    </label>
                                </div>
                                {if !xelatex_available {
                                    html! {
                                        <div class="modal-warning">
                                            {"⚠ XeLaTeX não está instalado. Para exportar PDF, instale o XeLaTeX."}
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                            </div>
                            <div class="modal-buttons">
                                <button class="modal-btn modal-btn-cancel" onclick={close_save_modal.clone()}>
                                    {"Cancelar"}
                                </button>
                                <button class="modal-btn modal-btn-save" onclick={handle_save.clone()}>
                                    {"Salvar"}
                                </button>
                            </div>
                        </div>
                    </div>
                }
            } else {
                html! {}
            }}

            {if state.show_settings_modal {
                let dispatch_for_close = dispatch.clone();
                let dispatch_for_save_file = dispatch.clone();
                let dispatch_for_dark_mode = dispatch.clone();
                let close_settings = Callback::from(move |_: MouseEvent| {
                    dispatch_for_close.reduce_mut(move |state| {
                        state.show_settings_modal = false;
                    });
                });
                let open_save_modal = Callback::from(move |_: MouseEvent| {
                    let state = dispatch_for_save_file.get();
                    if let Some(tab) = state.tabs.iter().find(|t| t.id == state.active_tab_id) {
                        dispatch_for_save_file.reduce_mut(move |state| {
                            state.show_settings_modal = false;
                            state.show_save_modal = true;
                            state.save_modal_filename = tab.title.clone();
                            state.save_modal_export_type = ExportType::Markdown;
                        });
                    }
                });
                let toggle_dark_mode = {
                    let dispatch = dispatch_for_dark_mode.clone();
                    Callback::from(move |_: MouseEvent| {
                        dispatch.reduce_mut(move |state| {
                            state.dark_mode = !state.dark_mode;
                        });
                    })
                };
                let is_dark = state.dark_mode;
                html! {
                    <div class="modal-overlay">
                        <div class="modal settings-modal">
                            <div class="modal-header">
                                {"Configurações"}
                                <button class="modal-close" onclick={close_settings}>{"×"}</button>
                            </div>
                            <div class="modal-body">
                                <div class="settings-option">
                                    <button class="settings-btn" onclick={open_save_modal}>
                                        {"exportar arquivo"}
                                    </button>
                                    <span class="settings-desc">{"exportar para .md ou .pdf em ABNT"}</span>
                                </div>
                                <div class="settings-divider"></div>
                                <div class="settings-option">
                                    <button class="settings-btn" onclick={toggle_dark_mode}>
                                        {if is_dark { "trocar para modo claro" } else { "trocar para modo escuro" }}
                                    </button>
                                    <span class="settings-desc">{"alternar tema do editor"}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                }
            } else {
                html! {}
            }}

            {if let Some(ref n) = state.notification.clone() {
                html! {
                    <div class={classes!("notification", if n.is_error { "notification-error" } else { "notification-success" })}>
                        <span>{&n.message}</span>
                        <button class="notification-close" onclick={dismiss_notification.clone()}>{"×"}</button>
                    </div>
                }
            } else {
                html! {}
            }}
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct BlockProps {
    pub block: Block,
    pub on_show_slash_menu: Callback<usize>,
    pub on_keydown: Callback<String>,
    pub on_backspace: Callback<usize>,
    pub on_delete: Callback<usize>,
    pub on_change: Callback<(usize, String)>,
    pub on_blur: Callback<(usize, String)>,
    pub on_enter: Callback<usize>,
    pub on_up_arrow: Callback<usize>,
    pub on_down_arrow: Callback<usize>,
    pub on_focus: Callback<usize>,
    pub focused_block_id: Option<usize>,
}

#[function_component(BlockComponent)]
pub fn block_component(props: &BlockProps) -> Html {
    let content_ref = use_node_ref();

    let oninput = {
        let on_show_slash_menu = props.on_show_slash_menu.clone();
        let on_change = props.on_change.clone();
        let block_id = props.block.id;
        Callback::from(move |e: InputEvent| {
            if let Some(target) = e.target_dyn_into::<web_sys::HtmlElement>() {
                let text = target.text_content().unwrap_or_default();
                if let Some(input_data) = e.data() {
                    if input_data == "/" {
                        on_show_slash_menu.emit(block_id);
                    }
                }
                on_change.emit((block_id, text));
            }
        })
    };

    let onblur = {
        let on_blur = props.on_blur.clone();
        let block_id = props.block.id;
        let content_ref = content_ref.clone();
        Callback::from(move |_: FocusEvent| {
            if let Some(element) = content_ref.cast::<web_sys::HtmlElement>() {
                let text = element.text_content().unwrap_or_default();
                on_blur.emit((block_id, text));
            }
        })
    };

    let onkeydown = {
        let on_enter = props.on_enter.clone();
        let on_up_arrow = props.on_up_arrow.clone();
        let on_down_arrow = props.on_down_arrow.clone();
        let on_backspace = props.on_backspace.clone();
        let on_delete = props.on_delete.clone();
        let on_keydown = props.on_keydown.clone();
        let block_id = props.block.id;
        let block_content = props.block.content.clone();
        let on_change = props.on_change.clone();
        let block_id_for_change = props.block.id;
        let content_ref_for_change = content_ref.clone();
        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();
            if key == "Enter" {
                if e.shift_key() {
                    let block_id = block_id_for_change;
                    let on_change = on_change.clone();
                    let content_ref = content_ref_for_change.clone();
                    let _ = gloo_timers::callback::Timeout::new(5, move || {
                        if let Some(element) = content_ref.cast::<web_sys::HtmlElement>() {
                            let text = element.text_content().unwrap_or_default();
                            on_change.emit((block_id, text));
                        }
                    });
                } else {
                    e.prevent_default();
                    on_enter.emit(block_id);
                }
            } else if key == "ArrowUp" {
                if let Some(selection) =
                    web_sys::window().and_then(|w| w.get_selection().ok().flatten())
                {
                    if let Ok(range) = selection.get_range_at(0) {
                        if let Ok(start) = range.start_offset() {
                            let start = start as usize;
                            if let Some(node) = range.start_container().ok() {
                                if let Some(content) = node.text_content() {
                                    if content[..start.min(content.len())].contains('\n') {
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
                e.prevent_default();
                on_up_arrow.emit(block_id);
            } else if key == "ArrowDown" {
                if let Some(selection) =
                    web_sys::window().and_then(|w| w.get_selection().ok().flatten())
                {
                    if let Ok(range) = selection.get_range_at(0) {
                        if let Ok(start) = range.start_offset() {
                            let start = start as usize;
                            if let Some(node) = range.start_container().ok() {
                                if let Some(content) = node.text_content() {
                                    if start < content.len() && content[start..].contains('\n') {
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
                e.prevent_default();
                on_down_arrow.emit(block_id);
            } else if key == "Backspace" {
                if block_content.is_empty() {
                    e.prevent_default();
                    on_backspace.emit(block_id);
                }
            } else if key == "Delete" {
                if block_content.is_empty() {
                    e.prevent_default();
                    on_delete.emit(block_id);
                }
            } else if key == "ArrowLeft" || key == "ArrowRight" {
                // Allow default cursor movement
            } else {
                on_keydown.emit(key);
            }
        })
    };

    let block_id = props.block.id;
    {
        let content_ref = content_ref.clone();
        let content = props.block.content.clone();
        let focused_id = props.focused_block_id;
        use_effect(move || {
            if focused_id != Some(block_id) {
                if let Some(element) = content_ref.cast::<web_sys::HtmlElement>() {
                    let current = element.text_content().unwrap_or_default();
                    if content.is_empty() {
                        if !current.is_empty() {
                            element.set_text_content(Some(&content));
                        }
                    } else if current.is_empty() {
                        element.set_text_content(Some(&content));
                    }
                }
            }
            || {}
        });
    }

    {
        let content_ref = content_ref.clone();
        let focused_id = props.focused_block_id;
        let block_id = props.block.id;
        use_effect(move || {
            if focused_id == Some(block_id) {
                if let Some(element) = content_ref.cast::<web_sys::HtmlElement>() {
                    element.focus().ok();
                }
            }
            || {}
        });
    }

    let show_placeholder = props.block.id == 0 && matches!(props.block.block_type, BlockType::Paragraph);
    let placeholder = if show_placeholder { "Type / for commands, or start writing" } else { "" };

    let block_type_class = match props.block.block_type {
        BlockType::Paragraph => "block-paragraph",
        BlockType::Heading1 => "block-heading-1",
        BlockType::Heading2 => "block-heading-2",
        BlockType::Heading3 => "block-heading-3",
        BlockType::BulletList => "block-bullet-list",
        BlockType::NumberedList => "block-numbered-list",
        BlockType::Quote => "block-quote",
        BlockType::CodeBlock => "block-code",
        BlockType::Image => "block-image",
        BlockType::HorizontalRule => "block-hr",
        BlockType::Citation => "block-citation",
        BlockType::Introducao => "block-introducao",
        BlockType::Desenvolvimento => "block-desenvolvimento",
        BlockType::Conclusao => "block-conclusao",
        BlockType::Teorema => "block-teorema",
        BlockType::Prova => "block-prova",
        BlockType::Definicao => "block-definicao",
        BlockType::Exemplo => "block-exemplo",
        BlockType::Observacao => "block-observacao",
        BlockType::CitacaoLonga => "block-citacao-longa",
    };

    html! {
        <div class={classes!("block", block_type_class)}>
            <div
                ref={content_ref}
                class="block-content"
                contenteditable="true"
                data-placeholder={placeholder}
                oninput={oninput}
                onblur={onblur}
                onkeydown={onkeydown}
            />
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SlashMenuProps {
    pub categories: Vec<SlashCategory>,
    pub on_select: Callback<Option<BlockType>>,
    pub on_close: Callback<()>,
}

#[function_component(SlashMenu)]
pub fn slash_menu(props: &SlashMenuProps) -> Html {
    let selected_category = use_state(|| 0usize);
    let selected_index = use_state(|| 0usize);

    let on_click = {
        let on_select = props.on_select.clone();
        let on_close = props.on_close.clone();
        Callback::from(move |block_type: Option<BlockType>| {
            on_select.emit(block_type);
            on_close.emit(());
        })
    };

    let categories = &props.categories;
    let total_categories = categories.len();
    let active_cat = *selected_category;

    {
        let selected_cat = selected_category.clone();
        let selected_idx = selected_index.clone();
        let on_close = props.on_close.clone();
        let on_select = props.on_select.clone();
        let cats = props.categories.clone();
        let tot_cats = total_categories;

        use_effect(move || {
            let handle_keydown = move |e: web_sys::KeyboardEvent| match e.key().as_str() {
                "ArrowRight" => {
                    e.prevent_default();
                    let new_cat = (*selected_cat + 1) % tot_cats;
                    selected_cat.set(new_cat);
                    selected_idx.set(0);
                }
                "ArrowLeft" => {
                    e.prevent_default();
                    let new_cat = if *selected_cat == 0 { tot_cats - 1 } else { *selected_cat - 1 };
                    selected_cat.set(new_cat);
                    selected_idx.set(0);
                }
                "ArrowDown" => {
                    e.prevent_default();
                    let len = cats[*selected_cat].options.len();
                    let new_val = (*selected_idx + 1) % len;
                    selected_idx.set(new_val);
                }
                "ArrowUp" => {
                    e.prevent_default();
                    let len = cats[*selected_cat].options.len();
                    let new_val = if *selected_idx == 0 { len - 1 } else { *selected_idx - 1 };
                    selected_idx.set(new_val);
                }
                "Enter" => {
                    e.prevent_default();
                    let options = &cats[*selected_cat].options;
                    if *selected_idx < options.len() {
                        if let Some(bt) = options[*selected_idx].block_type.clone() {
                            on_select.emit(Some(bt));
                            on_close.emit(());
                        }
                    }
                }
                "Escape" => {
                    on_close.emit(());
                }
                _ => {}
            };

            let closure =
                wasm_bindgen::closure::Closure::wrap(Box::new(handle_keydown) as Box<dyn Fn(_)>);

            if let Some(window) = web_sys::window() {
                window
                    .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
                    .ok();
            }
            closure.forget();

            || {}
        });
    }

    let set_category = {
        let selected_category = selected_category.clone();
        let selected_index = selected_index.clone();
        Callback::from(move |cat_idx: usize| {
            selected_category.set(cat_idx);
            selected_index.set(0);
        })
    };

    html! {
        <div class="slash-menu">
            <div class="slash-menu-categories">
                {for categories.iter().enumerate().map(|(i, cat)| {
                    let is_selected = *selected_category == i;
                    let set_cat = set_category.clone();
                    html! {
                        <div
                            class={classes!("slash-menu-cat", if is_selected { "selected" } else { "" })}
                            onmouseenter={move |_| set_cat.emit(i)}
                        >
                            {&cat.name}
                        </div>
                    }
                })}
            </div>
            <div class="slash-menu-items">
                <div class="slash-menu-items-inner">
                    {for categories[active_cat].options.iter().enumerate().map(|(i, option)| {
                        let is_selected = *selected_index == i;
                        let on_click = on_click.clone();
                        if let Some(ref bt) = option.block_type {
                            let option_block_type = bt.clone();
                            html! {
                                <div
                                    class={classes!("slash-menu-item", if is_selected { "selected" } else { "" })}
                                    onclick={move |_| on_click.emit(Some(option_block_type.clone()))}
                                >
                                    <span class="slash-menu-icon">{option.icon}</span>
                                    <span class="slash-menu-label">{&option.label}</span>
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    })}
                </div>
            </div>
        </div>
    }
}
