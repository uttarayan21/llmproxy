use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

mod components;
use components::{
    api_keys::ApiKeys, login::Login, platforms::Platforms, request_logs::RequestLogs,
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
    let is_authenticated = use_state(|| false);
    let is_checking_auth = use_state(|| true);

    // Check authentication on mount
    {
        let user = user.clone();
        let is_authenticated = is_authenticated.clone();
        let is_checking_auth = is_checking_auth.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(response) = Request::get("/api/user").send().await
                    && response.ok()
                    && let Ok(u) = response.json::<User>().await
                {
                    user.set(Some(u));
                    is_authenticated.set(true);
                }
                is_checking_auth.set(false);
            });
            || ()
        });
    }

    let on_login_success = {
        let user = user.clone();
        let is_authenticated = is_authenticated.clone();
        Callback::from(move |u: User| {
            user.set(Some(u));
            is_authenticated.set(true);
        })
    };

    let on_logout = {
        let user = user.clone();
        let is_authenticated = is_authenticated.clone();
        Callback::from(move |_| {
            let user = user.clone();
            let is_authenticated = is_authenticated.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let _ = Request::post("/api/auth/logout").send().await;
                user.set(None);
                is_authenticated.set(false);
            });
        })
    };

    // Show loading spinner while checking authentication
    if *is_checking_auth {
        return html! {
            <div class="min-h-screen flex items-center justify-center bg-gray-100">
                <div class="text-xl text-gray-600">{ "Loading..." }</div>
            </div>
        };
    }

    // Show login page if not authenticated
    if !*is_authenticated {
        return html! {
            <Login on_login_success={on_login_success} />
        };
    }

    // Show main app if authenticated
    let on_nav = |page: Page| {
        let current_page = current_page.clone();
        Callback::from(move |_| current_page.set(page.clone()))
    };

    html! {
        <div class="min-h-screen flex flex-col">
            <header class="bg-dark text-white px-8 py-4 flex justify-between items-center shadow-md">
                <h1 class="text-2xl font-semibold">{ "LLMPROXY" }</h1>
                <div class="flex items-center gap-4">
                    {
                        if let Some(u) = (*user).as_ref() {
                            html! {
                                <>
                                    <div class="text-sm">{ format!("User: {}", u.username) }</div>
                                    <button
                                        class="bg-white/20 text-white border border-white/30 px-4 py-2 rounded hover:bg-white/30 hover:border-white/50 transition"
                                        onclick={on_logout}
                                    >
                                        { "Logout" }
                                    </button>
                                </>
                            }
                        } else {
                            html! { <div class="text-sm">{ "Loading..." }</div> }
                        }
                    }
                </div>
            </header>

            <nav class="bg-white px-8 flex gap-0 border-b-2 border-gray-200">
                <button
                    class={if *current_page == Page::Logs {
                        "bg-none border-none px-6 py-4 cursor-pointer text-base text-primary border-b-3 border-primary hover:text-dark hover:bg-gray-50 transition"
                    } else {
                        "bg-none border-none px-6 py-4 cursor-pointer text-base text-gray-600 border-b-3 border-transparent hover:text-dark hover:bg-gray-50 transition"
                    }}
                    onclick={on_nav(Page::Logs)}
                >
                    { "Request Logs" }
                </button>
                <button
                    class={if *current_page == Page::Platforms {
                        "bg-none border-none px-6 py-4 cursor-pointer text-base text-primary border-b-3 border-primary hover:text-dark hover:bg-gray-50 transition"
                    } else {
                        "bg-none border-none px-6 py-4 cursor-pointer text-base text-gray-600 border-b-3 border-transparent hover:text-dark hover:bg-gray-50 transition"
                    }}
                    onclick={on_nav(Page::Platforms)}
                >
                    { "LLM Platforms" }
                </button>
                <button
                    class={if *current_page == Page::ApiKeys {
                        "bg-none border-none px-6 py-4 cursor-pointer text-base text-primary border-b-3 border-primary hover:text-dark hover:bg-gray-50 transition"
                    } else {
                        "bg-none border-none px-6 py-4 cursor-pointer text-base text-gray-600 border-b-3 border-transparent hover:text-dark hover:bg-gray-50 transition"
                    }}
                    onclick={on_nav(Page::ApiKeys)}
                >
                    { "API Keys" }
                </button>
            </nav>

            <main class="flex-1 p-8 max-w-screen-2xl w-full mx-auto">
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
