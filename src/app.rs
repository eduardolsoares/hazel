use wasm_bindgen::JsCast;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Tab {
    pub id: usize,
    pub name: String,
    pub title: String,
    pub blocks: Vec<Block>,
}

#[derive(Clone, PartialEq)]
pub struct Block {
    pub id: usize,
    pub block_type: BlockType,
    pub content: String,
}

#[derive(Clone, PartialEq)]
pub enum BlockType {
    Paragraph,
    Image,
}

#[derive(Clone, PartialEq)]
pub struct SlashOption {
    pub block_type: BlockType,
    pub label: String,
    pub icon: &'static str,
}

#[derive(Clone)]
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
        Self {
            tabs: vec![Tab {
                id: 0,
                name: "documento.tex".to_string(),
                title: "Untitled".to_string(),
                blocks: vec![Block {
                    id: 0,
                    block_type: BlockType::Paragraph,
                    content: String::new(),
                }],
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
            block_type: BlockType::Image,
            label: "Image".to_string(),
            icon: "🖼",
        },
    ]
}

#[function_component(App)]
pub fn app() -> Html {
    let state = use_state(|| EditorState::new());

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
            new_state.tabs.push(Tab {
                id: new_id,
                name: "novo_documento.tex".to_string(),
                title: "Untitled".to_string(),
                blocks: vec![Block {
                    id: new_state.next_block_id,
                    block_type: BlockType::Paragraph,
                    content: String::new(),
                }],
            });
            new_state.next_block_id += 1;
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
                {if let Some(tab) = active_tab {
                    let state_for_blocks = state.clone();
                    html! {
                        <div class="page">
                            <div class="page-title" contenteditable="true">
                                {"Untitled"}
                            </div>

                            <div class="blocks">
                                {for tab.blocks.iter().map(|block| {
                                    let is_menu_target = menu_block_id == Some(block.id);
                                    let block_id = block.id;
                                    let state_for_slash = state_for_blocks.clone();
                                    let state_for_keydown = state_for_blocks.clone();
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
}

#[function_component(BlockComponent)]
pub fn block_component(props: &BlockProps) -> Html {
    let oninput = {
        let on_slash_detected = props.on_slash_detected.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(target) = e.target_dyn_into::<web_sys::HtmlElement>() {
                if let Some(text) = target.text_content() {
                    if text.contains('/') {
                        on_slash_detected.emit(());
                    }
                }
            }
        })
    };

    let onkeydown = {
        let on_keydown = props.on_keydown.clone();
        Callback::from(move |e: KeyboardEvent| {
            on_keydown.emit(e.key());
        })
    };

    let placeholder = match props.block.block_type {
        BlockType::Paragraph => "Type / for commands, or start writing",
        BlockType::Image => "Click to upload or drag and drop",
    };

    let block_type_class = match props.block.block_type {
        BlockType::Paragraph => "block-paragraph",
        BlockType::Image => "block-image",
    };

    html! {
        <div class={classes!("block", block_type_class)}>
            <div
                class="block-content"
                contenteditable="true"
                data-placeholder={placeholder}
                oninput={oninput}
                onkeydown={onkeydown}
            >
            </div>
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
