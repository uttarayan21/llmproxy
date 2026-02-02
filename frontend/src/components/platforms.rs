use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct LlmPlatform {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub base_url: String,
    pub platform_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CreatePlatformRequest {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub platform_type: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UpdatePlatformRequest {
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub platform_type: String,
}

#[function_component(Platforms)]
pub fn platforms() -> Html {
    let platforms = use_state(Vec::<LlmPlatform>::new);
    let loading = use_state(|| true);
    let show_form = use_state(|| false);
    let editing_id = use_state(|| None::<i64>);
    let refresh = use_state(|| 0);

    let name_ref = use_node_ref();
    let base_url_ref = use_node_ref();
    let api_key_ref = use_node_ref();
    let platform_type_ref = use_node_ref();

    {
        let platforms = platforms.clone();
        let loading = loading.clone();
        let refresh_val = *refresh;
        use_effect_with(refresh_val, move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(response) = Request::get("/api/platforms").send().await
                    && let Ok(data) = response.json::<Vec<LlmPlatform>>().await {
                        platforms.set(data);
                    }
                loading.set(false);
            });
            || ()
        });
    }

    let on_toggle_form = {
        let show_form = show_form.clone();
        let editing_id = editing_id.clone();
        let name_ref = name_ref.clone();
        let base_url_ref = base_url_ref.clone();
        let api_key_ref = api_key_ref.clone();
        let platform_type_ref = platform_type_ref.clone();
        
        Callback::from(move |_| {
            let new_state = !*show_form;
            show_form.set(new_state);
            editing_id.set(None);
            
            // Clear form fields when closing or opening
            if let Some(input) = name_ref.cast::<HtmlInputElement>() {
                input.set_value("");
            }
            if let Some(input) = base_url_ref.cast::<HtmlInputElement>() {
                input.set_value("");
            }
            if let Some(input) = api_key_ref.cast::<HtmlInputElement>() {
                input.set_value("");
            }
            if let Some(input) = platform_type_ref.cast::<HtmlInputElement>() {
                input.set_value("");
            }
        })
    };

    let on_edit = {
        let show_form = show_form.clone();
        let editing_id = editing_id.clone();
        let name_ref = name_ref.clone();
        let base_url_ref = base_url_ref.clone();
        let api_key_ref = api_key_ref.clone();
        let platform_type_ref = platform_type_ref.clone();
        
        move |platform: LlmPlatform| {
            let show_form = show_form.clone();
            let editing_id = editing_id.clone();
            let name_ref = name_ref.clone();
            let base_url_ref = base_url_ref.clone();
            let api_key_ref = api_key_ref.clone();
            let platform_type_ref = platform_type_ref.clone();
            
            Callback::from(move |_| {
                editing_id.set(Some(platform.id));
                show_form.set(true);
                
                // Pre-fill form with existing values
                if let Some(input) = name_ref.cast::<HtmlInputElement>() {
                    input.set_value(&platform.name);
                }
                if let Some(input) = base_url_ref.cast::<HtmlInputElement>() {
                    input.set_value(&platform.base_url);
                }
                if let Some(input) = platform_type_ref.cast::<HtmlInputElement>() {
                    input.set_value(&platform.platform_type);
                }
                // Clear API key field (can't pre-fill since we don't have it)
                if let Some(input) = api_key_ref.cast::<HtmlInputElement>() {
                    input.set_value("");
                }
            })
        }
    };

    let on_submit = {
        let name_ref = name_ref.clone();
        let base_url_ref = base_url_ref.clone();
        let api_key_ref = api_key_ref.clone();
        let platform_type_ref = platform_type_ref.clone();
        let show_form = show_form.clone();
        let editing_id = editing_id.clone();
        let refresh = refresh.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let name = name_ref.cast::<HtmlInputElement>().unwrap().value();
            let base_url = base_url_ref.cast::<HtmlInputElement>().unwrap().value();
            let api_key = api_key_ref.cast::<HtmlInputElement>().unwrap().value();
            let platform_type = platform_type_ref
                .cast::<HtmlInputElement>()
                .unwrap()
                .value();

            let show_form = show_form.clone();
            let editing_id_val = *editing_id;
            let refresh = refresh.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let result = if let Some(id) = editing_id_val {
                    // Update existing platform
                    let request = UpdatePlatformRequest {
                        name,
                        base_url,
                        api_key: if api_key.is_empty() { None } else { Some(api_key) },
                        platform_type,
                    };
                    Request::put(&format!("/api/platforms/{}", id))
                        .json(&request)
                        .unwrap()
                        .send()
                        .await
                } else {
                    // Create new platform
                    let request = CreatePlatformRequest {
                        name,
                        base_url,
                        api_key,
                        platform_type,
                    };
                    Request::post("/api/platforms")
                        .json(&request)
                        .unwrap()
                        .send()
                        .await
                };

                if result.is_ok() {
                    show_form.set(false);
                    refresh.set(*refresh + 1);
                }
            });
        })
    };

    let on_delete = |id: i64| {
        let refresh = refresh.clone();
        Callback::from(move |_| {
            let refresh = refresh.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if Request::delete(&format!("/api/platforms/{}", id))
                    .send()
                    .await
                    .is_ok()
                {
                    refresh.set(*refresh + 1);
                }
            });
        })
    };

    html! {
        <div class="platforms">
            <div class="header-row">
                <h2>{ "LLM Platforms" }</h2>
                <button class="btn-primary" onclick={on_toggle_form}>
                    { if *show_form { "Cancel" } else { "Add Platform" } }
                </button>
            </div>

            {
                if *show_form {
                    let is_editing = editing_id.is_some();
                    html! {
                        <form class="platform-form" onsubmit={on_submit}>
                            <div class="form-group">
                                <label>{ "Name:" }</label>
                                <input type="text" ref={name_ref.clone()} required=true />
                            </div>
                            <div class="form-group">
                                <label>{ "Base URL:" }</label>
                                <input type="text" ref={base_url_ref.clone()} placeholder="https://api.openai.com/v1" required=true />
                            </div>
                            <div class="form-group">
                                <label>{ "API Key:" }</label>
                                <input 
                                    type="password" 
                                    ref={api_key_ref.clone()} 
                                    placeholder={if is_editing { "Leave empty to keep current key" } else { "" }}
                                    required={!is_editing} 
                                />
                            </div>
                            <div class="form-group">
                                <label>{ "Platform Type:" }</label>
                                <input type="text" ref={platform_type_ref.clone()} placeholder="openai, ollama, custom" required=true />
                            </div>
                            <button type="submit" class="btn-primary">
                                { if editing_id.is_some() { "Update" } else { "Create" } }
                            </button>
                        </form>
                    }
                } else {
                    html! {}
                }
            }

            {
                if *loading {
                    html! { <div class="loading">{ "Loading platforms..." }</div> }
                } else if platforms.is_empty() {
                    html! { <div class="empty">{ "No platforms configured yet" }</div> }
                } else {
                    html! {
                        <div class="platforms-list">
                            {
                                platforms.iter().map(|platform| {
                                    let on_delete_click = on_delete(platform.id);
                                    let on_edit_click = on_edit(platform.clone());
                                    html! {
                                        <div class="platform-item">
                                            <div class="platform-info">
                                                <h3>{ &platform.name }</h3>
                                                <div class="platform-details">
                                                    <div><strong>{ "URL:" }</strong> { &platform.base_url }</div>
                                                    <div><strong>{ "Type:" }</strong> { &platform.platform_type }</div>
                                                    <div><strong>{ "Created:" }</strong> { &platform.created_at }</div>
                                                </div>
                                            </div>
                                            <div class="platform-actions">
                                                <button class="btn-secondary" onclick={on_edit_click}>{ "Edit" }</button>
                                                <button class="btn-danger" onclick={on_delete_click}>{ "Delete" }</button>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Html>()
                            }
                        </div>
                    }
                }
            }
        </div>
    }
}
