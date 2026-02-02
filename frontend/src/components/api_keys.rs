use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct ProxyApiKey {
    pub id: i64,
    pub user_id: i64,
    pub key_prefix: String,
    pub name: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub id: i64,
    pub key: String,
    pub key_prefix: String,
    pub name: String,
    pub created_at: String,
}

#[function_component(ApiKeys)]
pub fn api_keys() -> Html {
    let keys = use_state(|| Vec::<ProxyApiKey>::new());
    let loading = use_state(|| true);
    let show_form = use_state(|| false);
    let new_key = use_state(|| Option::<ApiKeyResponse>::None);
    let refresh = use_state(|| 0);

    let name_ref = use_node_ref();

    {
        let keys = keys.clone();
        let loading = loading.clone();
        let refresh_val = *refresh;
        use_effect_with(refresh_val, move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(response) = Request::get("/api/keys").send().await {
                    if let Ok(data) = response.json::<Vec<ProxyApiKey>>().await {
                        keys.set(data);
                    }
                }
                loading.set(false);
            });
            || ()
        });
    }

    let on_toggle_form = {
        let show_form = show_form.clone();
        let new_key = new_key.clone();
        Callback::from(move |_| {
            show_form.set(!*show_form);
            new_key.set(None);
        })
    };

    let on_submit = {
        let name_ref = name_ref.clone();
        let show_form = show_form.clone();
        let new_key = new_key.clone();
        let refresh = refresh.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let name = name_ref.cast::<HtmlInputElement>().unwrap().value();

            let request = CreateApiKeyRequest { name };

            let show_form = show_form.clone();
            let new_key = new_key.clone();
            let refresh = refresh.clone();

            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(response) = Request::post("/api/keys")
                    .json(&request)
                    .unwrap()
                    .send()
                    .await
                {
                    if let Ok(key_response) = response.json::<ApiKeyResponse>().await {
                        new_key.set(Some(key_response));
                        show_form.set(false);
                        refresh.set(*refresh + 1);
                    }
                }
            });
        })
    };

    let on_delete = |id: i64| {
        let refresh = refresh.clone();
        Callback::from(move |_| {
            let refresh = refresh.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(_) = Request::delete(&format!("/api/keys/{}", id)).send().await {
                    refresh.set(*refresh + 1);
                }
            });
        })
    };

    let on_close_key_display = {
        let new_key = new_key.clone();
        Callback::from(move |_| {
            new_key.set(None);
        })
    };

    html! {
        <div class="api-keys">
            <div class="header-row">
                <h2>{ "Proxy API Keys" }</h2>
                <button class="btn-primary" onclick={on_toggle_form}>
                    { if *show_form { "Cancel" } else { "Generate Key" } }
                </button>
            </div>

            {
                if let Some(key) = (*new_key).as_ref() {
                    html! {
                        <div class="key-display">
                            <div class="key-display-header">
                                <h3>{ "New API Key Generated" }</h3>
                                <button onclick={on_close_key_display}>{ "Close" }</button>
                            </div>
                            <div class="key-warning">
                                <strong>{ "Important:" }</strong>
                                { " Copy this key now. You won't be able to see it again!" }
                            </div>
                            <div class="key-value">
                                <code>{ &key.key }</code>
                            </div>
                            <div class="key-info">
                                <div><strong>{ "Name:" }</strong> { &key.name }</div>
                                <div><strong>{ "Prefix:" }</strong> { &key.key_prefix }</div>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }

            {
                if *show_form {
                    html! {
                        <form class="api-key-form" onsubmit={on_submit}>
                            <div class="form-group">
                                <label>{ "Key Name:" }</label>
                                <input type="text" ref={name_ref.clone()} placeholder="My API Key" required=true />
                            </div>
                            <button type="submit" class="btn-primary">{ "Generate" }</button>
                        </form>
                    }
                } else {
                    html! {}
                }
            }

            {
                if *loading {
                    html! { <div class="loading">{ "Loading API keys..." }</div> }
                } else if keys.is_empty() {
                    html! { <div class="empty">{ "No API keys generated yet" }</div> }
                } else {
                    html! {
                        <div class="keys-list">
                            {
                                keys.iter().map(|key| {
                                    let on_delete_click = on_delete(key.id);
                                    html! {
                                        <div class="key-item">
                                            <div class="key-info">
                                                <h3>{ &key.name }</h3>
                                                <div class="key-details">
                                                    <div><strong>{ "Prefix:" }</strong> <code>{ &key.key_prefix }{ "..." }</code></div>
                                                    <div><strong>{ "Created:" }</strong> { &key.created_at }</div>
                                                    {
                                                        if let Some(last_used) = &key.last_used_at {
                                                            html! { <div><strong>{ "Last Used:" }</strong> { last_used }</div> }
                                                        } else {
                                                            html! { <div><strong>{ "Last Used:" }</strong> { "Never" }</div> }
                                                        }
                                                    }
                                                </div>
                                            </div>
                                            <button class="btn-danger" onclick={on_delete_click}>{ "Revoke" }</button>
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
