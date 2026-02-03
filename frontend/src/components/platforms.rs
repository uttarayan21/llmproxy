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
                        api_key: if api_key.is_empty() {
                            None
                        } else {
                            Some(api_key)
                        },
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
        <div>
            <div class="flex justify-between items-center mb-6">
                <h2 class="text-2xl text-dark">{ "LLM Platforms" }</h2>
                <button
                    class="bg-primary text-white px-4 py-2 border-none rounded cursor-pointer text-sm transition hover:bg-primary-dark"
                    onclick={on_toggle_form}
                >
                    { if *show_form { "Cancel" } else { "Add Platform" } }
                </button>
            </div>

            {
                if *show_form {
                    let is_editing = editing_id.is_some();
                    html! {
                        <form class="bg-white p-6 rounded-lg mb-6 shadow-md" onsubmit={on_submit}>
                            <div class="mb-4">
                                <label class="block mb-2 font-medium text-gray-700">{ "Name:" }</label>
                                <input
                                    type="text"
                                    ref={name_ref.clone()}
                                    required=true
                                    class="w-full px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-primary"
                                />
                            </div>
                            <div class="mb-4">
                                <label class="block mb-2 font-medium text-gray-700">{ "Base URL:" }</label>
                                <input
                                    type="text"
                                    ref={base_url_ref.clone()}
                                    placeholder="https://api.openai.com/v1"
                                    required=true
                                    class="w-full px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-primary"
                                />
                            </div>
                            <div class="mb-4">
                                <label class="block mb-2 font-medium text-gray-700">{ "API Key:" }</label>
                                <input
                                    type="password"
                                    ref={api_key_ref.clone()}
                                    placeholder={if is_editing { "Leave empty to keep current key" } else { "" }}
                                    required={!is_editing}
                                    class="w-full px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-primary"
                                />
                            </div>
                            <div class="mb-4">
                                <label class="block mb-2 font-medium text-gray-700">{ "Platform Type:" }</label>
                                <input
                                    type="text"
                                    ref={platform_type_ref.clone()}
                                    placeholder="openai, ollama, custom"
                                    required=true
                                    class="w-full px-2 py-2 border border-gray-300 rounded text-base focus:outline-none focus:border-primary"
                                />
                            </div>
                            <button
                                type="submit"
                                class="bg-primary text-white px-4 py-2 border-none rounded cursor-pointer text-sm transition hover:bg-primary-dark"
                            >
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
                    html! { <div class="text-center p-12 text-gray-600">{ "Loading platforms..." }</div> }
                } else if platforms.is_empty() {
                    html! { <div class="text-center p-12 text-gray-600">{ "No platforms configured yet" }</div> }
                } else {
                    html! {
                        <div class="grid gap-4">
                            {
                                platforms.iter().map(|platform| {
                                    let on_delete_click = on_delete(platform.id);
                                    let on_edit_click = on_edit(platform.clone());
                                    html! {
                                        <div class="bg-white p-6 rounded-lg shadow-md flex justify-between items-center">
                                            <div class="flex-1">
                                                <h3 class="text-dark mb-2">{ &platform.name }</h3>
                                                <div class="text-gray-600 text-sm">
                                                    <div class="my-1"><strong>{ "URL:" }</strong> { " " }{ &platform.base_url }</div>
                                                    <div class="my-1"><strong>{ "Type:" }</strong> { " " }{ &platform.platform_type }</div>
                                                    <div class="my-1"><strong>{ "Created:" }</strong> { " " }{ &platform.created_at }</div>
                                                </div>
                                            </div>
                                            <div class="flex gap-2">
                                                <button
                                                    class="bg-gray-500 text-white px-4 py-2 border-none rounded cursor-pointer text-sm transition hover:bg-gray-600"
                                                    onclick={on_edit_click}
                                                >
                                                    { "Edit" }
                                                </button>
                                                <button
                                                    class="bg-danger text-white px-4 py-2 border-none rounded cursor-pointer text-sm transition hover:bg-danger-dark"
                                                    onclick={on_delete_click}
                                                >
                                                    { "Delete" }
                                                </button>
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
