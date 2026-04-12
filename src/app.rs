use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yewdux::prelude::*;
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["__TAURI__", "core"], js_name = invoke)]
    fn invoke(cmd: &str, args: JsValue) -> js_sys::Promise;
}

fn save_markdown_invoke(content: String, file_path: Option<String>, default_name: Option<String>) {
    web_sys::console::log_1(
        &format!("Saving content: '{}', length: {}", content, content.len()).into(),
    );

    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "content": content,
        "filePath": file_path,
        "defaultName": default_name
    }))
    .unwrap();

    let promise = invoke("save_markdown", args.into());

    let _ = promise.then(&wasm_bindgen::closure::Closure::wrap(
        Box::new(move |result: JsValue| {
            web_sys::console::log_1(&format!("Save result: {:?}", result).into());
        }) as Box<dyn FnMut(JsValue)>,
    ));
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
}

#[derive(Clone, PartialEq)]
pub struct SlashOption {
    pub block_type: BlockType,
    pub label: String,
    pub icon: &'static str,
}

use yewdux::prelude::*;

#[derive(Clone, PartialEq, Store)]
pub struct EditorState {
    pub tabs: Vec<Tab>,
    pub active_tab_id: usize,
    pub next_tab_id: usize,
    pub next_block_id: usize,
    pub show_slash_menu: bool,
    pub slash_menu_block_id: Option<usize>,
    pub focused_block_id: Option<usize>,
}

impl Default for EditorState {
    fn default() -> Self {
        let mut buffer = Buffer::new();
        buffer.push_back(Block::new(0, BlockType::Paragraph));

        Self {
            tabs: vec![Tab {
                id: 0,
                name: "Untitled.md".to_string(),
                title: "Untitled".to_string(),
                buffer,
                file_path: None,
                is_dirty: false,
                block_order: vec![0],
            }],
            active_tab_id: 0,
            next_tab_id: 1,
            next_block_id: 1,
            show_slash_menu: false,
            slash_menu_block_id: None,
            focused_block_id: None,
        }
    }
}

fn get_slash_options() -> Vec<SlashOption> {
    vec![
        SlashOption {
            block_type: BlockType::Paragraph,
            label: "Paragraph".to_string(),
            icon: "¶",
        },
        SlashOption {
            block_type: BlockType::Heading1,
            label: "Heading 1".to_string(),
            icon: "H1",
        },
        SlashOption {
            block_type: BlockType::Heading2,
            label: "Heading 2".to_string(),
            icon: "H2",
        },
        SlashOption {
            block_type: BlockType::Heading3,
            label: "Heading 3".to_string(),
            icon: "H3",
        },
        SlashOption {
            block_type: BlockType::BulletList,
            label: "Bullet List".to_string(),
            icon: "•",
        },
        SlashOption {
            block_type: BlockType::NumberedList,
            label: "Numbered List".to_string(),
            icon: "1.",
        },
        SlashOption {
            block_type: BlockType::Quote,
            label: "Quote".to_string(),
            icon: "❝",
        },
        SlashOption {
            block_type: BlockType::CodeBlock,
            label: "Code Block".to_string(),
            icon: "</>",
        },
        SlashOption {
            block_type: BlockType::Image,
            label: "Image".to_string(),
            icon: "🖼",
        },
        SlashOption {
            block_type: BlockType::HorizontalRule,
            label: "Horizontal Rule".to_string(),
            icon: "—",
        },
    ]
}

#[function_component(App)]
pub fn app() -> Html {
    let (state, dispatch) = use_store::<EditorState>();

    let save_callback = {
        let dispatch = dispatch.clone();
        Callback::from(move |_| {
            let state = dispatch.get();
            web_sys::console::log_1(&format!("dumping all state").into());

            for (tab_idx, tab) in state.tabs.iter().enumerate() {
                web_sys::console::log_1(
                    &format!(
                        "Tab[{}] - id: {}, is_active: {}",
                        tab_idx,
                        tab.id,
                        tab.id == state.active_tab_id
                    )
                    .into(),
                );
                for (bid, block) in &tab.buffer.blocks {
                    web_sys::console::log_1(
                        &format!("    Block id: {}, content: '{}'", bid, block.content).into(),
                    );
                }
            }

            if let Some(tab) = state.tabs.iter().find(|t| t.id == state.active_tab_id) {
                let content = tab.buffer.to_markdown();
                web_sys::console::log_1(&format!("Content to save: '{}'", content).into());
                let file_path = tab.file_path.clone();
                let default_name = if tab.file_path.is_none() {
                    Some(format!("{}.md", tab.title.replace(' ', "_")))
                } else {
                    None
                };
                save_markdown_invoke(content, file_path, default_name);
            }
        })
    };

    use_effect(move || {
        use gloo_events::EventListener;
        use std::sync::OnceLock;

        static REGISTERED: OnceLock<()> = OnceLock::new();

        if REGISTERED.get().is_none() {
            let save = save_callback.clone();
            let _ = REGISTERED.set(());

            let listener = EventListener::new(
                &web_sys::window().unwrap().unchecked_ref(),
                "keydown",
                move |event| {
                    let e = event.unchecked_ref::<web_sys::KeyboardEvent>();
                    if e.ctrl_key() && e.key() == "s" {
                        e.prevent_default();
                        save.emit(());
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
                    name: "Untitled.md".to_string(),
                    title: "Untitled".to_string(),
                    buffer,
                    file_path: None,
                    is_dirty: false,
                    block_order: vec![block_id],
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
        Callback::from(move |block_type: BlockType| {
            dispatch.reduce_mut(move |state| {
                if let Some(block_id) = state.slash_menu_block_id {
                    if let Some(tab) = state.tabs.iter_mut().find(|t| t.id == state.active_tab_id) {
                        if let Some(block) = tab.buffer.blocks.get_mut(&block_id) {
                            block.block_type = block_type.clone();
                            block.content = String::new();
                        }
                    }
                }
                state.show_slash_menu = false;
                state.slash_menu_block_id = None;
            });
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
        <div class="app">
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
                                <span class="tab-name">{&tab.name}</span>
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
                                        let name = if text.is_empty() { "Untitled".to_string() } else { text.clone() };
                                        dispatch_for_title.reduce_mut(move |state| {
                                            if let Some(t) = state.tabs.iter_mut().find(|t| t.id == tab_id_for_title) {
                                                t.title = text;
                                                t.name = format!("{}.md", name.replace(' ', "_"));
                                            }
                                        });
                                    }
                                })}
                                onblur={Callback::from(move |e: FocusEvent| {
                                    if let Some(target) = e.target_dyn_into::<web_sys::HtmlElement>() {
                                        let text = target.text_content().unwrap_or_default();
                                        let name = if text.is_empty() { "Untitled".to_string() } else { text.clone() };
                                        dispatch_for_title_blur.reduce_mut(move |state| {
                                            if let Some(t) = state.tabs.iter_mut().find(|t| t.id == tab_id_for_title) {
                                                t.title = text;
                                                t.name = format!("{}.md", name.replace(' ', "_"));
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
                                                        }
                                                    });
                                                })}
                                                on_blur={let dispatch2 = dispatch_clone.clone(); Callback::from(move |(id, content): (usize, String)| {
                                                    dispatch2.reduce_mut(move |state| {
                                                        if let Some(tab) = state.tabs.iter_mut().find(|t| t.id == state.active_tab_id) {
                                                            if let Some(block) = tab.buffer.blocks.get_mut(&id) {
                                                                block.content = content;
                                                            }
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
                                                        options={get_slash_options()}
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
        let on_blur = props.on_blur.clone();
        let block_id = props.block.id;
        Callback::from(move |e: InputEvent| {
            if let Some(target) = e.target_dyn_into::<web_sys::HtmlElement>() {
                let text = target.text_content().unwrap_or_default();
                if let Some(input_data) = e.data() {
                    if input_data == "/" {
                        on_show_slash_menu.emit(block_id);
                    } else {
                        on_change.emit((block_id, text));
                    }
                }
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
            } else {
                on_keydown.emit(key);
            }
        })
    };

    let block_id = props.block.id;
    {
        let content_ref = content_ref.clone();
        let content = props.block.content.clone();
        let block_type = props.block.block_type.clone();
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

    let placeholder = match props.block.block_type {
        BlockType::Paragraph => "Type / for commands, or start writing",
        BlockType::Heading1 => "Heading 1",
        BlockType::Heading2 => "Heading 2",
        BlockType::Heading3 => "Heading 3",
        BlockType::BulletList => "Bullet list item",
        BlockType::NumberedList => "Numbered list item",
        BlockType::Quote => "Quote",
        BlockType::CodeBlock => "Code",
        BlockType::Image => "Image URL",
        BlockType::HorizontalRule => "",
        BlockType::Citation => "Citation reference",
    };

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
    pub options: Vec<SlashOption>,
    pub on_select: Callback<BlockType>,
    pub on_close: Callback<()>,
}

#[function_component(SlashMenu)]
pub fn slash_menu(props: &SlashMenuProps) -> Html {
    let selected_index = use_state(|| 0usize);

    let on_click = {
        let on_select = props.on_select.clone();
        let on_close = props.on_close.clone();
        Callback::from(move |block_type: BlockType| {
            on_select.emit(block_type);
            on_close.emit(());
        })
    };

    {
        let selected = selected_index.clone();
        let options_len = props.options.len();
        let on_close = props.on_close.clone();
        let on_select = props.on_select.clone();
        let options = props.options.clone();

        use_effect(move || {
            let handle_keydown = move |e: web_sys::KeyboardEvent| match e.key().as_str() {
                "ArrowDown" => {
                    e.prevent_default();
                    let new_val = (*selected + 1) % options_len;
                    selected.set(new_val);
                }
                "ArrowUp" => {
                    e.prevent_default();
                    let new_val = if *selected == 0 {
                        options_len - 1
                    } else {
                        *selected - 1
                    };
                    selected.set(new_val);
                }
                "Tab" => {
                    e.prevent_default();
                    let new_val = if e.shift_key() {
                        if *selected == 0 {
                            options_len - 1
                        } else {
                            *selected - 1
                        }
                    } else {
                        (*selected + 1) % options_len
                    };
                    selected.set(new_val);
                }
                "Enter" => {
                    e.prevent_default();
                    let current_idx = *selected;
                    if current_idx < options.len() {
                        on_select.emit(options[current_idx].block_type.clone());
                        on_close.emit(());
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

    html! {
        <div class="slash-menu">
            <div class="slash-menu-header">{"Basic blocks"}</div>
            {for props.options.iter().enumerate().map(|(i, option)| {
                let is_selected = *selected_index == i;
                let option_block_type = option.block_type.clone();
                let on_click = on_click.clone();
                html! {
                    <div
                        class={classes!("slash-menu-item", if is_selected { "selected" } else { "" })}
                        onclick={move |_| on_click.emit(option_block_type.clone())}
                    >
                        <span class="slash-menu-icon">{option.icon}</span>
                        <span class="slash-menu-label">{&option.label}</span>
                    </div>
                }
            })}
        </div>
    }
}
