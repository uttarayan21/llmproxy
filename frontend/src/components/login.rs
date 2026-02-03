use crate::User;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Properties, PartialEq)]
pub struct LoginProps {
    pub on_login_success: Callback<User>,
}

#[function_component(Login)]
pub fn login(props: &LoginProps) -> Html {
    let username_ref = use_node_ref();
    let password_ref = use_node_ref();
    let error = use_state(|| Option::<String>::None);
    let loading = use_state(|| false);
    let is_register_mode = use_state(|| false);

    let on_submit = {
        let username_ref = username_ref.clone();
        let password_ref = password_ref.clone();
        let error = error.clone();
        let loading = loading.clone();
        let is_register_mode = is_register_mode.clone();
        let on_login_success = props.on_login_success.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let username_input = username_ref.cast::<HtmlInputElement>();
            let password_input = password_ref.cast::<HtmlInputElement>();

            if let Some(username_el) = username_input
                && let Some(password_el) = password_input
            {
                let username = username_el.value();
                let password = password_el.value();

                if username.is_empty() || password.is_empty() {
                    error.set(Some("Username and password are required".to_string()));
                    return;
                }

                let error = error.clone();
                let loading = loading.clone();
                let is_register = *is_register_mode;
                let on_login_success = on_login_success.clone();

                loading.set(true);
                error.set(None);

                spawn_local(async move {
                    let endpoint = if is_register {
                        "/api/auth/register"
                    } else {
                        "/api/auth/login"
                    };

                    let request_body = LoginRequest {
                        username: username.clone(),
                        password: password.clone(),
                    };

                    let result = Request::post(endpoint)
                        .json(&request_body)
                        .unwrap()
                        .send()
                        .await;

                    loading.set(false);

                    match result {
                        Ok(response) => {
                            if response.ok() {
                                if let Ok(user) = response.json::<User>().await {
                                    on_login_success.emit(user);
                                } else {
                                    error.set(Some("Failed to parse response".to_string()));
                                }
                            } else {
                                let status = response.status();
                                let error_text = response
                                    .text()
                                    .await
                                    .unwrap_or_else(|_| "Unknown error".to_string());

                                // Try to parse error JSON
                                if let Ok(error_json) =
                                    serde_json::from_str::<serde_json::Value>(&error_text)
                                {
                                    if let Some(message) =
                                        error_json.get("message").and_then(|m| m.as_str())
                                    {
                                        error.set(Some(format!("Error {}: {}", status, message)));
                                    } else {
                                        error
                                            .set(Some(format!("Error {}: {}", status, error_text)));
                                    }
                                } else {
                                    error.set(Some(format!("Error {}: {}", status, error_text)));
                                }
                            }
                        }
                        Err(e) => {
                            error.set(Some(format!("Network error: {}", e)));
                        }
                    }
                });
            }
        })
    };

    let toggle_mode = {
        let is_register_mode = is_register_mode.clone();
        let error = error.clone();
        Callback::from(move |_| {
            is_register_mode.set(!*is_register_mode);
            error.set(None);
        })
    };

    html! {
        <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-[#667eea] to-[#764ba2] p-4">
            <div class="bg-white p-10 rounded-xl shadow-2xl w-full max-w-md">
                <h1 class="text-dark text-center mb-2 text-3xl">{ "LLMPROXY" }</h1>
                <h2 class="text-gray-600 text-center mb-8 text-xl font-normal">
                    { if *is_register_mode { "Create Account" } else { "Login" } }
                </h2>

                <form onsubmit={on_submit} class="p-0 shadow-none mb-4">
                    <div class="mb-6">
                        <label for="username" class="block mb-2 font-semibold text-dark">{ "Username" }</label>
                        <input
                            ref={username_ref}
                            type="text"
                            id="username"
                            name="username"
                            placeholder="Enter username"
                            disabled={*loading}
                            required={true}
                            class="w-full px-3 py-3 text-base border-2 border-gray-200 rounded focus:outline-none focus:border-[#667eea] focus:shadow-[0_0_0_3px_rgba(102,126,234,0.1)]"
                        />
                    </div>

                    <div class="mb-6">
                        <label for="password" class="block mb-2 font-semibold text-dark">{ "Password" }</label>
                        <input
                            ref={password_ref}
                            type="password"
                            id="password"
                            name="password"
                            placeholder="Enter password"
                            disabled={*loading}
                            required={true}
                            class="w-full px-3 py-3 text-base border-2 border-gray-200 rounded focus:outline-none focus:border-[#667eea] focus:shadow-[0_0_0_3px_rgba(102,126,234,0.1)]"
                        />
                    </div>

                    {
                        if let Some(err) = (*error).as_ref() {
                            html! {
                                <div class="bg-red-50 text-red-700 px-3 py-3 rounded-md mb-4 text-sm border border-red-200">
                                    { err }
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }

                    <button
                        type="submit"
                        disabled={*loading}
                        class="w-full px-3 py-3 bg-gradient-to-br from-[#667eea] to-[#764ba2] text-white text-base font-semibold border-none rounded-md cursor-pointer transition-all hover:translate-y-[-2px] hover:shadow-[0_5px_15px_rgba(102,126,234,0.4)] disabled:opacity-60 disabled:cursor-not-allowed disabled:transform-none"
                    >
                        { if *loading && *is_register_mode {
                            "Creating account..."
                        } else if *loading {
                            "Logging in..."
                        } else if *is_register_mode {
                            "Create Account"
                        } else {
                            "Login"
                        }}
                    </button>
                </form>

                <div class="text-center mt-6">
                    <button
                        type="button"
                        onclick={toggle_mode}
                        class="bg-none border-none text-[#667eea] cursor-pointer text-sm px-2 py-2 underline hover:text-[#764ba2]"
                    >
                        { if *is_register_mode {
                            "Already have an account? Login"
                        } else {
                            "Don't have an account? Register"
                        }}
                    </button>
                </div>
            </div>
        </div>
    }
}
