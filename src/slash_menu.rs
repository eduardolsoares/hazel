use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

use crate::models::{BlockType, SlashCategory};

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
                    let new_cat = if *selected_cat == 0 {
                        tot_cats - 1
                    } else {
                        *selected_cat - 1
                    };
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
                    let new_val = if *selected_idx == 0 {
                        len - 1
                    } else {
                        *selected_idx - 1
                    };
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

            let closure = Closure::wrap(Box::new(handle_keydown) as Box<dyn Fn(_)>);

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
