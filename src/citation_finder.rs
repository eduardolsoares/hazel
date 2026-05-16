use gloo_timers::callback::Timeout;
use js_sys;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures;
use web_sys;
use yew::prelude::*;

#[derive(Clone, Debug)]
pub struct Work {
    pub doi: Option<String>,
    pub title: String,
    pub authors: Vec<String>,
    pub year: Option<String>,
    pub journal: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub publisher: Option<String>,
    pub work_type: String,
}

fn format_abnt(work: &Work) -> String {
    let authors_str = if work.authors.is_empty() {
        String::new()
    } else {
        let mut s = String::new();
        for (i, author) in work.authors.iter().enumerate() {
            if i > 0 {
                s.push_str("; ");
            }
            s.push_str(author);
        }
        s
    };

    let cap_author = if authors_str.is_empty() {
        String::new()
    } else {
        format!("{}. ", authors_str)
    };

    let title = work.title.trim().trim_end_matches('.');
    let year = work.year.as_deref().unwrap_or("s.d.");

    match work.work_type.as_str() {
        "journal-article" | "journal-issue" => {
            let journal = work.journal.as_deref().unwrap_or("");
            let vol = work.volume.as_deref().unwrap_or("");
            let iss = work.issue.as_deref().unwrap_or("");
            let pg = work.pages.as_deref().unwrap_or("");

            let mut parts = vec![format!("{}{}.", cap_author, title)];

            if !journal.is_empty() {
                parts.push(format!(" *{}*", journal));
            }

            let mut details = Vec::new();
            if !vol.is_empty() {
                details.push(format!("v. {}", vol));
            }
            if !iss.is_empty() {
                details.push(format!("n. {}", iss));
            }
            if !pg.is_empty() {
                details.push(format!("p. {}", pg));
            }
            if !details.is_empty() {
                parts.push(format!(", {}", details.join(", ")));
            }

            parts.push(format!(", {}.", year));
            parts.concat()
        }
        "book" | "monograph" | "book-section" | "book-chapter" | "edited-book" => {
            let publisher = work.publisher.as_deref().unwrap_or("");
            let mut parts = vec![format!("{}{}.", cap_author, title)];
            if !publisher.is_empty() {
                parts.push(format!(" {};", publisher));
            }
            parts.push(format!(" {}.", year));
            parts.concat()
        }
        "proceedings-article" | "paper-conference" => {
            let journal = work.journal.as_deref().unwrap_or("");
            let publisher = work.publisher.as_deref().unwrap_or("");
            let pg = work.pages.as_deref().unwrap_or("");
            let vol = work.volume.as_deref().unwrap_or("");

            let mut parts = vec![format!("{}{}.", cap_author, title)];
            if !journal.is_empty() {
                parts.push(format!(" In: {}", journal));
            }
            if !vol.is_empty() {
                parts.push(format!(", v. {}", vol));
            }
            if !pg.is_empty() {
                parts.push(format!(", p. {}", pg));
            }
            if !publisher.is_empty() {
                parts.push(format!(" {};", publisher));
            }
            parts.push(format!(" {}.", year));
            parts.concat()
        }
        "report" | "report-series" => {
            let publisher = work.publisher.as_deref().unwrap_or("");
            let mut parts = vec![format!("{}{}.", cap_author, title)];
            if !publisher.is_empty() {
                parts.push(format!(" {};", publisher));
            }
            parts.push(format!(" {}.", year));
            parts.concat()
        }
        "dissertation" | "thesis" => {
            let publisher = work.publisher.as_deref().unwrap_or("");
            let mut parts = vec![format!("{}{}.", cap_author, title)];
            if !publisher.is_empty() {
                parts.push(format!(" {};", publisher));
            }
            parts.push(format!(" {}.", year));
            parts.concat()
        }
        _ => {
            let publisher = work.publisher.as_deref().unwrap_or("");
            let mut parts = vec![format!("{}{}.", cap_author, title)];
            if !publisher.is_empty() {
                parts.push(format!(" {};", publisher));
            }
            parts.push(format!(" {}.", year));
            parts.concat()
        }
    }
}

fn parse_work(value: &js_sys::Object) -> Option<Work> {
    let doi = js_sys::Reflect::get(value, &JsValue::from_str("DOI"))
        .ok()
        .and_then(|v| v.as_string());

    let title = js_sys::Reflect::get(value, &JsValue::from_str("title"))
        .ok()
        .and_then(|v| {
            if let Some(arr) = v.dyn_ref::<js_sys::Array>() {
                arr.get(0).as_string()
            } else {
                v.as_string()
            }
        })
        .unwrap_or_default();

    let authors = js_sys::Reflect::get(value, &JsValue::from_str("author"))
        .ok()
        .and_then(|v| {
            let arr = v.dyn_ref::<js_sys::Array>()?;
            let mut result = Vec::new();
            for i in 0..arr.length() {
                let item = arr.get(i);
                if let Some(obj) = item.dyn_ref::<js_sys::Object>() {
                    let family = js_sys::Reflect::get(obj, &JsValue::from_str("family"))
                        .ok()
                        .and_then(|v| v.as_string())
                        .unwrap_or_default();
                    let given = js_sys::Reflect::get(obj, &JsValue::from_str("given"))
                        .ok()
                        .and_then(|v| v.as_string())
                        .unwrap_or_default();
                    if !family.is_empty() {
                        result.push(format!("{}, {}", family.to_uppercase(), given));
                    }
                }
            }
            Some(result)
        })
        .unwrap_or_default();

    let year = js_sys::Reflect::get(value, &JsValue::from_str("published-print"))
        .ok()
        .or_else(|| {
            js_sys::Reflect::get(value, &JsValue::from_str("published-online")).ok()
        })
        .or_else(|| {
            js_sys::Reflect::get(value, &JsValue::from_str("issued")).ok()
        })
        .and_then(|v| {
            let obj = v.dyn_ref::<js_sys::Object>()?;
            let parts = js_sys::Reflect::get(obj, &JsValue::from_str("date-parts"))
                .ok()?;
            let arr = parts.dyn_ref::<js_sys::Array>()?;
            let first = arr.get(0);
            let inner = first.dyn_ref::<js_sys::Array>()?;
            inner.get(0).as_string()
        });

    let journal = js_sys::Reflect::get(value, &JsValue::from_str("container-title"))
        .ok()
        .and_then(|v| {
            if let Some(arr) = v.dyn_ref::<js_sys::Array>() {
                arr.get(0).as_string()
            } else {
                v.as_string()
            }
        });

    let volume = js_sys::Reflect::get(value, &JsValue::from_str("volume"))
        .ok()
        .and_then(|v| v.as_string());

    let issue = js_sys::Reflect::get(value, &JsValue::from_str("issue"))
        .ok()
        .and_then(|v| v.as_string());

    let pages = js_sys::Reflect::get(value, &JsValue::from_str("page"))
        .ok()
        .and_then(|v| v.as_string());

    let publisher = js_sys::Reflect::get(value, &JsValue::from_str("publisher"))
        .ok()
        .and_then(|v| v.as_string());

    let work_type = js_sys::Reflect::get(value, &JsValue::from_str("type"))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default();

    Some(Work {
        doi,
        title,
        authors,
        year,
        journal,
        volume,
        issue,
        pages,
        publisher,
        work_type,
    })
}

async fn fetch_works(query: &str) -> Result<Vec<Work>, JsValue> {
    let url = format!(
        "https://api.crossref.org/works?query={}&rows=10&mailto=user@example.com",
        js_sys::encode_uri_component(query)
    );

    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(web_sys::RequestMode::Cors);

    let request = web_sys::Request::new_with_str_and_init(&url, &opts)?;

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp_value.dyn_into()?;

    if !resp.ok() {
        return Err(JsValue::from_str(&format!("HTTP error: {}", resp.status())));
    }

    let json = wasm_bindgen_futures::JsFuture::from(resp.json()?).await?;
    let obj = json.dyn_ref::<js_sys::Object>().ok_or_else(|| JsValue::from_str("not an object"))?;

    let msg = js_sys::Reflect::get(obj, &JsValue::from_str("message"))?;
    let msg_obj = msg.dyn_ref::<js_sys::Object>().ok_or_else(|| JsValue::from_str("no message"))?;

    let items = js_sys::Reflect::get(msg_obj, &JsValue::from_str("items"))?;
    let items_arr = items.dyn_ref::<js_sys::Array>().ok_or_else(|| JsValue::from_str("no items"))?;

    let mut works = Vec::new();
    for i in 0..items_arr.length() {
        let item = items_arr.get(i);
        if let Some(obj) = item.dyn_ref::<js_sys::Object>() {
            if let Some(work) = parse_work(obj) {
                works.push(work);
            }
        }
    }

    Ok(works)
}

#[derive(Properties, PartialEq)]
pub struct CitationFinderProps {
    pub on_insert: Callback<String>,
    pub on_close: Callback<()>,
}

#[function_component(CitationFinder)]
pub fn citation_finder(props: &CitationFinderProps) -> Html {
    let query = use_state(|| String::new());
    let results = use_state(|| Vec::<Work>::new());
    let loading = use_state(|| false);
    let selected_index = use_state(|| 0usize);
    let has_searched = use_state(|| false);

    let debounce_timer = use_mut_ref(|| Option::<Timeout>::None);

    let on_search = {
        let query = query.clone();
        let results = results.clone();
        let loading = loading.clone();
        let has_searched = has_searched.clone();
        let selected_index = selected_index.clone();
        let debounce_timer = debounce_timer.clone();

        Callback::from(move |new_query: String| {
            query.set(new_query.clone());

            if let Some(timer) = debounce_timer.borrow_mut().take() {
                timer.cancel();
            }

            let results_c = results.clone();
            let loading_c = loading.clone();
            let has_searched_c = has_searched.clone();
            let selected_index_c = selected_index.clone();

            if new_query.trim().is_empty() {
                results_c.set(Vec::new());
                loading_c.set(false);
                return;
            }

            loading_c.set(true);
            has_searched_c.set(true);

            let timer = Timeout::new(350, move || {
                selected_index_c.set(0);
                wasm_bindgen_futures::spawn_local(async move {
                    match fetch_works(&new_query).await {
                        Ok(works) => {
                            results_c.set(works);
                        }
                        Err(_) => {
                            results_c.set(Vec::new());
                        }
                    }
                    loading_c.set(false);
                });
            });
            *debounce_timer.borrow_mut() = Some(timer);
        })
    };

    let on_insert = props.on_insert.clone();
    let on_close = props.on_close.clone();

    let insert_citation = {
        let results = results.clone();
        let on_insert = on_insert.clone();
        let on_close = on_close.clone();
        Callback::from(move |idx: usize| {
            if idx < results.len() {
                let citation = format_abnt(&results[idx]);
                on_insert.emit(citation);
                on_close.emit(());
            }
        })
    };

    let onkeydown = {
        let selected_index = selected_index.clone();
        let results_len = results.len();
        let insert_citation = insert_citation.clone();
        let on_close = on_close.clone();
        Callback::from(move |e: KeyboardEvent| {
            match e.key().as_str() {
                "ArrowDown" => {
                    e.prevent_default();
                    let len = results_len;
                    if len > 0 {
                        let new_idx = (*selected_index + 1) % len;
                        selected_index.set(new_idx);
                    }
                }
                "ArrowUp" => {
                    e.prevent_default();
                    let len = results_len;
                    if len > 0 {
                        let new_idx = if *selected_index == 0 {
                            len - 1
                        } else {
                            *selected_index - 1
                        };
                        selected_index.set(new_idx);
                    }
                }
                "Enter" => {
                    e.prevent_default();
                    let len = results_len;
                    if len > 0 && *selected_index < len {
                        insert_citation.emit(*selected_index);
                    }
                }
                "Escape" => {
                    e.prevent_default();
                    on_close.emit(());
                }
                _ => {}
            }
        })
    };

    let input_ref = use_node_ref();

    {
        let input_ref = input_ref.clone();
        use_effect(move || {
            if let Some(input) = input_ref.cast::<web_sys::HtmlInputElement>() {
                input.focus().ok();
            }
            || {}
        });
    }

    html! {
        <div class="citation-finder-overlay" onclick={let c = on_close.clone(); Callback::from(move |_: MouseEvent| c.emit(()))}>
            <div class="citation-finder" onclick={Callback::from(move |e: MouseEvent| e.stop_propagation())}>
                <div class="citation-finder-header">
                    <span class="citation-finder-title">{"Buscar referência"}</span>
                    <button class="citation-finder-close" onclick={let c = on_close.clone(); Callback::from(move |_: MouseEvent| c.emit(()))}>{"×"}</button>
                </div>
                <div class="citation-finder-search">
                    <input
                        ref={input_ref}
                        type="text"
                        class="citation-finder-input"
                        placeholder="Digite o nome do artigo, autor, DOI..."
                        value={(*query).clone()}
                        oninput={let on_search = on_search.clone(); Callback::from(move |e: InputEvent| {
                            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                on_search.emit(input.value());
                            }
                        })}
                        onkeydown={onkeydown}
                    />
                    {if *loading {
                        html! { <span class="citation-finder-spinner">{"..."}</span> }
                    } else {
                        html! {}
                    }}
                </div>
                <div class="citation-finder-results">
                    {if *has_searched && results.is_empty() && !*loading {
                        html! {
                            <div class="citation-finder-empty">
                                {"Nenhum resultado encontrado."}
                            </div>
                        }
                    } else {
                        html! {}
                    }}
                    {for results.iter().enumerate().map(|(i, work)| {
                        let is_selected = i == *selected_index;
                        let onclick = {
                            let insert_citation = insert_citation.clone();
                            Callback::from(move |_| insert_citation.emit(i))
                        };
                        let authors = work.authors.join("; ");
                        let journal = work.journal.as_deref().unwrap_or("");
                        let year = work.year.as_deref().unwrap_or("");
                        html! {
                            <div class={classes!("citation-finder-item", if is_selected { "selected" } else { "" })}
                                onclick={onclick}
                                onmouseenter={let selected_index = selected_index.clone(); Callback::from(move |_| selected_index.set(i))}
                            >
                                <div class="citation-finder-item-title">
                                    {&work.title}
                                </div>
                                <div class="citation-finder-item-meta">
                                    {if !authors.is_empty() {
                                        html! { <span class="citation-finder-item-authors">{authors}</span> }
                                    } else {
                                        html! {}
                                    }}
                                    {if !journal.is_empty() {
                                        html! { <span class="citation-finder-item-journal">{journal}</span> }
                                    } else {
                                        html! {}
                                    }}
                                    {if !year.is_empty() {
                                        html! { <span class="citation-finder-item-year">{"("}{year}{")"}</span> }
                                    } else {
                                        html! {}
                                    }}
                                </div>
                            </div>
                        }
                    })}
                </div>
            </div>
        </div>
    }
}
