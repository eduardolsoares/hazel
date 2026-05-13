use std::sync::OnceLock;

use gloo_events::EventListener;
use serde_json;
use serde_wasm_bindgen;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures;
use web_sys;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::block::BlockComponent;
use crate::ipc::*;
use crate::models::*;
use crate::slash_menu::SlashMenu;

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

    let _open_save_modal = {
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
                            if let Ok(result_obj) =
                                serde_wasm_bindgen::from_value::<serde_json::Value>(value)
                            {
                                let success = result_obj
                                    .get("success")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false);
                                web_sys::console::log_1(
                                    &format!("Save success: {}", success).into(),
                                );
                                if success {
                                    dispatch2.reduce_mut(move |state| {
                                        for tab in state.tabs.iter_mut() {
                                            let content = tab.buffer.to_markdown();
                                            web_sys::console::log_1(
                                                &format!(
                                                    "Setting saved_content, len={}",
                                                    content.len()
                                                )
                                                .into(),
                                            );
                                            tab.saved_content = Some(content);
                                            tab.is_dirty = false;
                                        }
                                    });
                                    web_sys::console::log_1(
                                        &"State saved, is_dirty cleared".into(),
                                    );
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
                                if let Ok(result_obj) =
                                    serde_wasm_bindgen::from_value::<serde_json::Value>(value)
                                {
                                    let success = result_obj
                                        .get("success")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false);
                                    let notification = if success {
                                        let path = result_obj
                                            .get("file_path")
                                            .and_then(|v| v.as_str())
                                            .map(|s| format!("Arquivo salvo: {}", s))
                                            .unwrap_or_else(|| "Salvo com sucesso".to_string());
                                        Notification {
                                            message: path,
                                            is_error: false,
                                        }
                                    } else {
                                        let error = result_obj
                                            .get("error")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("Erro ao salvar")
                                            .to_string();
                                        Notification {
                                            message: error,
                                            is_error: true,
                                        }
                                    };
                                    dispatch_for_notify.reduce_mut(move |state| {
                                        state.notification = Some(notification);
                                        if let Some(t) =
                                            state.tabs.iter_mut().find(|t| t.id == active_tab_id)
                                        {
                                            t.is_dirty = false;
                                        }
                                        state.show_save_modal = false;
                                    });
                                }
                            }
                            Err(_) => {}
                        }
                    });

                    // Close modal immediately, notification will show async
                } else if state.xelatex_available {
                    let default_name = Some(filename.replace(' ', "_"));

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
                                if let Ok(result_obj) =
                                    serde_wasm_bindgen::from_value::<serde_json::Value>(value)
                                {
                                    let success = result_obj
                                        .get("success")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false);
                                    web_sys::console::log_1(
                                        &format!("Export success: {}", success).into(),
                                    );

                                    let notification = if success {
                                        let path = result_obj
                                            .get("file_path")
                                            .and_then(|v| v.as_str())
                                            .map(|s| format!("PDF salvo em: {}", s))
                                            .unwrap_or_else(|| "PDF salvo com sucesso".to_string());
                                        Notification {
                                            message: path,
                                            is_error: false,
                                        }
                                    } else {
                                        let error = result_obj
                                            .get("error")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("Erro ao exportar PDF")
                                            .to_string();
                                        Notification {
                                            message: error,
                                            is_error: true,
                                        }
                                    };
                                    dispatch_for_notify.reduce_mut(move |state| {
                                        state.notification = Some(notification);
                                    });
                                }
                            }
                            Err(e) => {
                                web_sys::console::log_1(
                                    &format!("PDF export error: {:?}", e).into(),
                                );
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

    let _update_modal_filename = {
        let dispatch = dispatch.clone();
        Callback::from(move |filename: String| {
            dispatch.reduce_mut(move |state| {
                state.save_modal_filename = filename;
            });
        })
    };

    let _set_export_type = {
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
        let dispatch = dispatch.clone();
        static CHECKED: OnceLock<()> = OnceLock::new();

        if CHECKED.get().is_none() {
            let _ = CHECKED.set(());

            wasm_bindgen_futures::spawn_local(async move {
                let promise = check_xelatex_invoke();
                let result = match wasm_bindgen_futures::JsFuture::from(promise).await {
                    Ok(value) => value,
                    Err(_) => return,
                };

                if let Ok(result_obj) = serde_wasm_bindgen::from_value::<serde_json::Value>(result)
                {
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
                        if let Some(tab) =
                            state.tabs.iter_mut().find(|t| t.id == state.active_tab_id)
                        {
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
                    let is_same = saved
                        .as_ref()
                        .map(|s| s == &current_content)
                        .unwrap_or(false);
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
                    let is_same = saved
                        .as_ref()
                        .map(|s| s == &current_content)
                        .unwrap_or(false);
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
