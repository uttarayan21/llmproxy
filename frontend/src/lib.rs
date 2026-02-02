use yew::prelude::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

mod components;
use components::{
    request_logs::RequestLogs,
    platforms::Platforms,
    api_keys::ApiKeys,
};

#[derive(Clone, PartialEq)]
enum Page {
    Logs,
    Platforms,
    ApiKeys,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub created_at: String,
    pub updated_at: String,
}

#[function_component(App)]
fn app() -> Html {
    let current_page = use_state(|| Page::Logs);
    let user = use_state(|| Option::<User>::None);

    {
        let user = user.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(response) = Request::get("/api/user").send().await {
                    if let Ok(u) = response.json::<User>().await {
                        user.set(Some(u));
                    }
                }
            });
            || ()
        });
    }

    let on_nav = |page: Page| {
        let current_page = current_page.clone();
        Callback::from(move |_| current_page.set(page.clone()))
    };

    html! {
        <div class="app">
            <header class="header">
                <h1>{ "LLMPROXY" }</h1>
                {
                    if let Some(u) = (*user).as_ref() {
                        html! { <div class="user-info">{ format!("User: {}", u.username) }</div> }
                    } else {
                        html! { <div class="user-info">{ "Loading..." }</div> }
                    }
                }
            </header>

            <nav class="nav">
                <button
                    class={if *current_page == Page::Logs { "active" } else { "" }}
                    onclick={on_nav(Page::Logs)}
                >
                    { "Request Logs" }
                </button>
                <button
                    class={if *current_page == Page::Platforms { "active" } else { "" }}
                    onclick={on_nav(Page::Platforms)}
                >
                    { "LLM Platforms" }
                </button>
                <button
                    class={if *current_page == Page::ApiKeys { "active" } else { "" }}
                    onclick={on_nav(Page::ApiKeys)}
                >
                    { "API Keys" }
                </button>
            </nav>

            <main class="content">
                {
                    match *current_page {
                        Page::Logs => html! { <RequestLogs /> },
                        Page::Platforms => html! { <Platforms /> },
                        Page::ApiKeys => html! { <ApiKeys /> },
                    }
                }
            </main>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}
