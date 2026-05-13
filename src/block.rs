use gloo_timers::callback::Timeout;
use yew::prelude::*;

use crate::models::{Block, BlockType};

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
                    let _ = Timeout::new(5, move || {
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

    let show_placeholder =
        props.block.id == 0 && matches!(props.block.block_type, BlockType::Paragraph);
    let placeholder = if show_placeholder {
        "Type / for commands, or start writing"
    } else {
        ""
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
