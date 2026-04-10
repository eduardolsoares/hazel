use std::cell::RefCell;
use std::rc::Rc;

use js_sys::Object;
use js_sys::Reflect;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::console;
use yew::prelude::*;
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["__TAURI__", "core"], js_name = invoke)]
    fn invoke(cmd: &str, args: JsValue) -> js_sys::Promise;
}

fn save_markdown_invoke(content: String, file_path: Option<String>) {
    web_sys::console::log_1(
        &format!("Saving content: '{}', length: {}", content, content.len()).into(),
    );

    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "content": content,
        "filePath": file_path
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
}

#[derive(Clone, PartialEq)]
pub struct SlashOption {
    pub block_type: BlockType,
    pub label: String,
    pub icon: &'static str,
}

#[derive(Clone, PartialEq)]
pub struct EditorState {
    pub tabs: Vec<Tab>,
    pub active_tab_id: usize,
    pub next_tab_id: usize,
    pub next_block_id: usize,
    pub show_slash_menu: bool,
    pub slash_menu_block_id: Option<usize>,
}

impl EditorState {
    pub fn new() -> Self {
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
            }],
            active_tab_id: 0,
            next_tab_id: 1,
            next_block_id: 1,
            show_slash_menu: false,
            slash_menu_block_id: None,
        }
    }

    pub fn get_active_tab(&self) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id == self.active_tab_id)
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
    let state = use_state(|| EditorState::new());

    // Use a RefCell wrapped in Rc to share mutable state
    let state_holder =
        use_state(|| Rc::new(RefCell::new((*state).clone())) as Rc<RefCell<EditorState>>);

    // Update the holder when state changes
    {
        let state_holder = state_holder.clone();
        let state = state.clone();
        use_effect(move || {
            // Clone the state value into the holder when effect runs
            *state_holder.borrow_mut() = (*state).clone();
            || {}
        });
    }

    // Callback to update both state and holder
    let update_state = {
        let state = state.clone();
        let state_holder = state_holder.clone();
        Callback::from(move |new_state: EditorState| {
            state.set(new_state.clone());
            *state_holder.borrow_mut() = new_state;
        })
    };

    let save_callback = {
        let state_holder = state_holder.clone();
        Callback::from(move |_| {
            let current_state = (*state_holder.borrow()).clone();
            web_sys::console::log_1(&format!("dumping all state").into());

            for (tab_idx, tab) in current_state.tabs.iter().enumerate() {
                web_sys::console::log_1(
                    &format!(
                        "Tab[{}] - id: {}, is_active: {}",
                        tab_idx,
                        tab.id,
                        tab.id == current_state.active_tab_id
                    )
                    .into(),
                );
                for (bid, block) in &tab.buffer.blocks {
                    web_sys::console::log_1(
                        &format!("    Block id: {}, content: '{}'", bid, block.content).into(),
                    );
                }
            }

            if let Some(tab) = current_state
                .tabs
                .iter()
                .find(|t| t.id == current_state.active_tab_id)
            {
                let content = tab.buffer.to_markdown();
                web_sys::console::log_1(&format!("Content to save: '{}'", content).into());
                let file_path = tab.file_path.clone();
                save_markdown_invoke(content, file_path);
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
        let state = state.clone();
        Callback::from(move |tab_id: usize| {
            let mut new_state = (*state).clone();
            new_state.active_tab_id = tab_id;
            new_state.show_slash_menu = false;
            state.set(new_state);
        })
    };

    let add_tab = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut new_state = (*state).clone();
            let new_id = new_state.next_tab_id;
            new_state.next_tab_id += 1;

            let mut buffer = Buffer::new();
            let block_id = new_state.next_block_id;
            buffer.push_back(Block::new(block_id, BlockType::Paragraph));
            new_state.next_block_id += 1;

            new_state.tabs.push(Tab {
                id: new_id,
                name: "Untitled.md".to_string(),
                title: "Untitled".to_string(),
                buffer,
                file_path: None,
                is_dirty: false,
            });
            new_state.active_tab_id = new_id;
            state.set(new_state);
        })
    };

    let close_tab = {
        let state = state.clone();
        Callback::from(move |tab_id: usize| {
            let mut new_state = (*state).clone();
            if new_state.tabs.len() > 1 {
                new_state.tabs.retain(|t| t.id != tab_id);
                if new_state.active_tab_id == tab_id {
                    new_state.active_tab_id = new_state.tabs[0].id;
                }
            }
            state.set(new_state);
        })
    };

    let hide_slash_menu = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut new_state = (*state).clone();
            new_state.show_slash_menu = false;
            new_state.slash_menu_block_id = None;
            state.set(new_state);
        })
    };

    let select_slash_option = {
        let state = state.clone();
        Callback::from(move |_block_type: BlockType| {
            let mut new_state = (*state).clone();
            new_state.show_slash_menu = false;
            state.set(new_state);
        })
    };

    let active_tab = (*state).get_active_tab().cloned();
    let show_menu = state.show_slash_menu;
    let menu_block_id = state.slash_menu_block_id;

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
                    let state_for_blocks = state.clone();
                    let tab_id = tab.id;
                    html! {
                        <div class="page" key={tab_id}>
                            <div class="page-title" contenteditable="true">
                                {"Untitled"}
                            </div>

                            <div class="blocks">
                                {for tab.buffer.to_vec().iter().map(|block| {
                                    let is_menu_target = menu_block_id == Some(block.id);
                                    let block_id = block.id;
                                    let state_for_slash = state_for_blocks.clone();
                                    let state_for_keydown = state_for_blocks.clone();
                                    let state_for_change = state_for_blocks.clone();
                                    html! {
                                        <>
                                            <BlockComponent
                                                block={block.clone()}
                                                on_slash_detected={Callback::from(move |_| {
                                                    let mut ns = (*state_for_slash).clone();
                                                    ns.show_slash_menu = true;
                                                    ns.slash_menu_block_id = Some(block_id);
                                                    state_for_slash.set(ns);
                                                })}
                                                on_keydown={Callback::from(move |key: String| {
                                                    let mut ns = (*state_for_keydown).clone();
                                                    if key == "Backspace" {
                                                        ns.show_slash_menu = false;
                                                        ns.slash_menu_block_id = None;
                                                    }
                                                    state_for_keydown.set(ns);
                                                })}
                                            on_change={Callback::from(move |(id, content): (usize, String)| {
                                                    let mut ns = (*state_for_change).clone();
                                                    if let Some(tab) = ns.tabs.iter_mut().find(|t| t.id == ns.active_tab_id) {
                                                        if let Some(block) = tab.buffer.blocks.get_mut(&id) {
                                                            block.content = content;
                                                        }
                                                    }
                                                    state_for_change.set(ns);
                                                })}
                                            />
                                            {if show_menu && is_menu_target {
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
    pub on_slash_detected: Callback<()>,
    pub on_keydown: Callback<String>,
    pub on_change: Callback<(usize, String)>,
}

#[function_component(BlockComponent)]
pub fn block_component(props: &BlockProps) -> Html {
    let content_ref = use_node_ref();

    let oninput = {
        let on_slash_detected = props.on_slash_detected.clone();
        let on_change = props.on_change.clone();
        let block_id = props.block.id;
        Callback::from(move |e: InputEvent| {
            if let Some(target) = e.target_dyn_into::<web_sys::HtmlElement>() {
                let text = target.text_content().unwrap_or_default();
                if text.contains('/') {
                    on_slash_detected.emit(());
                }
                on_change.emit((block_id, text));
            }
        })
    };

    let onkeydown = {
        let on_keydown = props.on_keydown.clone();
        Callback::from(move |e: KeyboardEvent| {
            on_keydown.emit(e.key());
        })
    };

    let block_id = props.block.id;
    {
        let content_ref = content_ref.clone();
        let content = props.block.content.clone();
        use_effect(move || {
            if let Some(element) = content_ref.cast::<web_sys::HtmlElement>() {
                let current = element.text_content().unwrap_or_default();
                if current.is_empty() && !content.is_empty() {
                    element.set_text_content(Some(&content));
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
