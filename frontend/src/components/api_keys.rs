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
    pub llm_platform_id: Option<i64>,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub llm_platform_id: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub id: i64,
    pub key: String,
    pub key_prefix: String,
    pub name: String,
    pub llm_platform_id: i64,
    pub created_at: String,
}

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

#[function_component(ApiKeys)]
pub fn api_keys() -> Html {
    let keys = use_state(Vec::<ProxyApiKey>::new);
    let platforms = use_state(Vec::<LlmPlatform>::new);
    let loading = use_state(|| true);
    let show_form = use_state(|| false);
    let new_key = use_state(|| Option::<ApiKeyResponse>::None);
    let refresh = use_state(|| 0);

    let name_ref = use_node_ref();
    let platform_ref = use_node_ref();

    {
        let keys = keys.clone();
        let platforms = platforms.clone();
        let loading = loading.clone();
        let refresh_val = *refresh;
        use_effect_with(refresh_val, move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                // Fetch keys
                if let Ok(response) = Request::get("/api/keys").send().await
                    && let Ok(data) = response.json::<Vec<ProxyApiKey>>().await
                {
                    keys.set(data);
                }
                // Fetch platforms
                if let Ok(response) = Request::get("/api/platforms").send().await
                    && let Ok(data) = response.json::<Vec<LlmPlatform>>().await
                {
                    platforms.set(data);
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
        let platform_ref = platform_ref.clone();
        let show_form = show_form.clone();
        let new_key = new_key.clone();
        let refresh = refresh.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let name = name_ref.cast::<HtmlInputElement>().unwrap().value();
            let platform_id_str = platform_ref
                .cast::<web_sys::HtmlSelectElement>()
                .unwrap()
                .value();

            // Parse platform ID, return early if invalid
            let platform_id = match platform_id_str.parse::<i64>() {
                Ok(id) => id,
                Err(_) => {
                    // Invalid selection, don't submit
                    return;
                }
            };

            let request = CreateApiKeyRequest {
                name,
                llm_platform_id: platform_id,
            };

            let show_form = show_form.clone();
            let new_key = new_key.clone();
            let refresh = refresh.clone();

            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(response) = Request::post("/api/keys")
                    .json(&request)
                    .unwrap()
                    .send()
                    .await
                    && let Ok(key_response) = response.json::<ApiKeyResponse>().await
                {
                    new_key.set(Some(key_response));
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
                if Request::delete(&format!("/api/keys/{}", id))
                    .send()
                    .await
                    .is_ok()
                {
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
        <div>
            <div class="flex justify-between items-center mb-6">
                <h2 class="text-2xl text-dark">{ "Proxy API Keys" }</h2>
                <button
                    class="bg-primary text-white px-4 py-2 border-none rounded cursor-pointer text-sm transition hover:bg-primary-dark"
                    onclick={on_toggle_form}
                >
                    { if *show_form { "Cancel" } else { "Generate Key" } }
                </button>
            </div>

            {
                if let Some(key) = (*new_key).as_ref() {
                    html! {
                        <div class="bg-amber-50 border-2 border-amber-400 rounded-lg p-6 mb-6">
                            <div class="flex justify-between items-center mb-4">
                                <h3 class="text-dark">{ "New API Key Generated" }</h3>
                                <button
                                    onclick={on_close_key_display}
                                    class="bg-primary text-white px-4 py-2 border-none rounded cursor-pointer text-sm transition hover:bg-primary-dark"
                                >
                                    { "Close" }
                                </button>
                            </div>
                            <div class="text-amber-800 mb-4 px-3 py-3 bg-white rounded">
                                <strong>{ "Important:" }</strong>
                                { " Copy this key now. You won't be able to see it again!" }
                            </div>
                            <div class="bg-dark text-white p-4 rounded mb-4 overflow-x-auto">
                                <code class="font-mono text-sm">{ &key.key }</code>
                            </div>
                            <div>
                                <div><strong>{ "Name:" }</strong> { " " }{ &key.name }</div>
                                <div><strong>{ "Prefix:" }</strong> { " " }{ &key.key_prefix }</div>
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
                        <form class="bg-white p-6 rounded-lg mb-6 shadow-md" onsubmit={on_submit}>
                            <div class="mb-4">
                                <label class="block mb-2 font-medium text-gray-700">{ "Key Name:" }</label>
                                <input
                                    type="text"
                                    ref={name_ref.clone()}
                                    placeholder="My API Key"
                                    required=true
                                    class="w-full px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-primary"
                                />
                            </div>
                            <div class="mb-4">
                                <label class="block mb-2 font-medium text-gray-700">{ "Platform:" }</label>
                                <select
                                    ref={platform_ref.clone()}
                                    required=true
                                    class="w-full px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-primary"
                                >
                                    <option value="" disabled=true selected=true>{ "Select a platform" }</option>
                                    {
                                        platforms.iter().map(|platform| {
                                            html! {
                                                <option value={platform.id.to_string()}>
                                                    { &platform.name }{ " (" }{ &platform.platform_type }{ ")" }
                                                </option>
                                            }
                                        }).collect::<Html>()
                                    }
                                </select>
                            </div>
                            <button
                                type="submit"
                                class="bg-primary text-white px-4 py-2 border-none rounded cursor-pointer text-sm transition hover:bg-primary-dark"
                            >
                                { "Generate" }
                            </button>
                        </form>
                    }
                } else {
                    html! {}
                }
            }

            {
                if *loading {
                    html! { <div class="text-center p-12 text-gray-600">{ "Loading API keys..." }</div> }
                } else if keys.is_empty() {
                    html! { <div class="text-center p-12 text-gray-600">{ "No API keys generated yet" }</div> }
                } else {
                    html! {
                        <div class="grid gap-4">
                            {
                                keys.iter().map(|key| {
                                    let on_delete_click = on_delete(key.id);
                                    let platform_name = key.llm_platform_id
                                        .and_then(|id| platforms.iter().find(|p| p.id == id))
                                        .map(|p| p.name.clone())
                                        .unwrap_or_else(|| "Unknown".to_string());

                                    html! {
                                        <div class="bg-white p-6 rounded-lg shadow-md flex justify-between items-center">
                                            <div class="flex-1">
                                                <h3 class="text-dark mb-2">{ &key.name }</h3>
                                                <div class="text-gray-600 text-sm">
                                                    <div class="my-1"><strong>{ "Platform:" }</strong> { " " }{ platform_name }</div>
                                                    <div class="my-1">
                                                        <strong>{ "Prefix:" }</strong>
                                                        { " " }
                                                        <code class="bg-gray-100 px-2 py-1 rounded font-mono">
                                                            { &key.key_prefix }{ "..." }
                                                        </code>
                                                    </div>
                                                    <div class="my-1"><strong>{ "Created:" }</strong> { " " }{ &key.created_at }</div>
                                                    {
                                                        if let Some(last_used) = &key.last_used_at {
                                                            html! { <div class="my-1"><strong>{ "Last Used:" }</strong> { " " }{ last_used }</div> }
                                                        } else {
                                                            html! { <div class="my-1"><strong>{ "Last Used:" }</strong> { " Never" }</div> }
                                                        }
                                                    }
                                                </div>
                                            </div>
                                            <button
                                                class="bg-danger text-white px-4 py-2 border-none rounded cursor-pointer text-sm transition hover:bg-danger-dark"
                                                onclick={on_delete_click}
                                            >
                                                { "Revoke" }
                                            </button>
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
