use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use web_sys::window;
use yew::prelude::*;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestLog {
    pub id: i64,
    pub user_id: i64,
    pub proxy_api_key_id: i64,
    pub llm_platform_id: i64,
    pub method: String,
    pub path: String,
    pub request_headers: String,
    pub request_body: Option<String>,
    pub outgoing_url: Option<String>,
    pub outgoing_headers: Option<String>,
    pub outgoing_body: Option<String>,
    pub response_status: Option<i32>,
    pub response_headers: Option<String>,
    pub response_body: Option<String>,
    pub duration_ms: Option<i64>,
    pub error: Option<String>,
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

// Helper function to generate curl command for incoming request
fn generate_incoming_curl(log: &RequestLog, proxy_url: &str) -> String {
    // Ensure the path starts with a slash
    let path = if log.path.starts_with('/') {
        log.path.clone()
    } else {
        format!("/{}", log.path)
    };

    let mut curl = format!("curl -X {} '{}{}'", log.method, proxy_url, path);

    // Add headers
    if let Ok(headers) = serde_json::from_str::<HashMap<String, String>>(&log.request_headers) {
        for (key, value) in headers.iter() {
            // Skip host header as curl will add it
            if key.to_lowercase() != "host" {
                curl.push_str(&format!(" \\\n  -H '{}: {}'", key, value));
            }
        }
    }

    // Add body
    if let Some(body) = &log.request_body {
        curl.push_str(&format!(" \\\n  -d '{}'", body.replace('\'', "'\\''")));
    }

    curl
}

// Helper function to generate curl command for outgoing request
fn generate_outgoing_curl(log: &RequestLog) -> Option<String> {
    let url = log.outgoing_url.as_ref()?;
    let mut curl = format!("curl -X {} '{}'", log.method, url);

    // Add headers
    if let Some(headers_str) = &log.outgoing_headers
        && let Ok(headers) = serde_json::from_str::<HashMap<String, String>>(headers_str)
    {
        for (key, value) in headers.iter() {
            curl.push_str(&format!(" \\\n  -H '{}: {}'", key, value));
        }
    }

    // Add body
    if let Some(body) = &log.outgoing_body {
        curl.push_str(&format!(" \\\n  -d '{}'", body.replace('\'', "'\\''")));
    }

    Some(curl)
}

// Helper function to copy text to clipboard
fn copy_to_clipboard(text: &str) {
    if let Some(window) = window() {
        let clipboard = window.navigator().clipboard();
        let _ = clipboard.write_text(text);
    }
}

// Helper function to prettify JSON if valid, otherwise return original
fn prettify_json(text: &str) -> String {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| text.to_string())
    } else {
        text.to_string()
    }
}

#[function_component(RequestLogs)]
pub fn request_logs() -> Html {
    let logs = use_state(Vec::<RequestLog>::new);
    let platforms = use_state(Vec::<LlmPlatform>::new);
    let selected_log = use_state(|| Option::<RequestLog>::None);
    let loading = use_state(|| true);

    {
        let logs = logs.clone();
        let platforms = platforms.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                // Fetch logs
                if let Ok(response) = Request::get("/api/logs?limit=50").send().await
                    && let Ok(data) = response.json::<Vec<RequestLog>>().await
                {
                    logs.set(data);
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

    let on_select_log = {
        let selected_log = selected_log.clone();
        Callback::from(move |log: RequestLog| {
            selected_log.set(Some(log));
        })
    };

    let on_close_detail = {
        let selected_log = selected_log.clone();
        Callback::from(move |_| {
            selected_log.set(None);
        })
    };

    html! {
        <div>
            <h2 class="text-2xl text-dark mb-6">{ "Request Logs" }</h2>

            {
                if *loading {
                    html! { <div class="text-center p-12 text-gray-600">{ "Loading logs..." }</div> }
                } else if logs.is_empty() {
                    html! { <div class="text-center p-12 text-gray-600">{ "No request logs yet" }</div> }
                } else {
                    html! {
                        <div class="grid grid-cols-2 gap-6">
                            <div class="bg-white rounded-lg overflow-hidden shadow-md max-h-[80vh] overflow-y-auto">
                                {
                                    logs.iter().map(|log| {
                                        let log_clone = log.clone();
                                        let on_select = on_select_log.clone();
                                        let platform_name = platforms.iter()
                                            .find(|p| p.id == log.llm_platform_id)
                                            .map(|p| p.name.clone())
                                            .unwrap_or_else(|| "Unknown".to_string());

                                        html! {
                                            <div
                                                class="p-4 border-b border-gray-100 cursor-pointer transition hover:bg-gray-50"
                                                onclick={Callback::from(move |_| on_select.emit(log_clone.clone()))}
                                            >
                                                <div class="flex gap-2 items-center mb-2">
                                                    <span class="font-semibold text-primary text-sm">{ &log.method }</span>
                                                    <span class="flex-1 text-gray-700 text-sm">{ &log.path }</span>
                                                    {
                                                        if let Some(status) = log.response_status {
                                                            let status_class = match status / 100 {
                                                                2 => "bg-green-100 text-green-800",
                                                                4 | 5 => "bg-red-100 text-red-800",
                                                                _ => "bg-gray-100 text-gray-800",
                                                            };
                                                            html! {
                                                                <span class={format!("px-2 py-1 rounded text-xs font-semibold {}", status_class)}>
                                                                    { status }
                                                                </span>
                                                            }
                                                        } else {
                                                            html! {
                                                                <span class="px-2 py-1 rounded text-xs font-semibold bg-red-100 text-red-800">
                                                                    { "Error" }
                                                                </span>
                                                            }
                                                        }
                                                    }
                                                </div>
                                                <div class="flex gap-4 text-xs text-gray-500">
                                                    <span>{ format!("Platform: {}", platform_name) }</span>
                                                    {
                                                        if let Some(duration) = log.duration_ms {
                                                            html! { <span>{ format!("{}ms", duration) }</span> }
                                                        } else {
                                                            html! {}
                                                        }
                                                    }
                                                    <span>{ &log.created_at }</span>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Html>()
                                }
                            </div>

                            {
                                if let Some(log) = (*selected_log).as_ref() {
                                    let platform = platforms.iter()
                                        .find(|p| p.id == log.llm_platform_id)
                                        .cloned();

                                    // Prepare curl commands outside html! macro
                                    let proxy_url = window()
                                        .and_then(|w| w.location().origin().ok())
                                        .unwrap_or_else(|| "http://localhost:8080".to_string());
                                    let incoming_curl = generate_incoming_curl(log, &format!("{}/proxy", proxy_url));
                                    let outgoing_curl = generate_outgoing_curl(log);

                                    let on_copy_incoming = {
                                        let curl = incoming_curl.clone();
                                        Callback::from(move |_| {
                                            copy_to_clipboard(&curl);
                                        })
                                    };

                                    let on_copy_outgoing = {
                                        let curl = outgoing_curl.clone();
                                        Callback::from(move |_| {
                                            if let Some(c) = &curl {
                                                copy_to_clipboard(c);
                                            }
                                        })
                                    };

                                    html! {
                                        <div class="bg-white rounded-lg p-6 shadow-md max-h-[80vh] overflow-y-auto">
                                            <div class="flex justify-between items-center mb-6">
                                                <h3 class="text-dark">{ "Request Details" }</h3>
                                                <button
                                                    onclick={on_close_detail}
                                                    class="bg-primary text-white px-4 py-2 border-none rounded cursor-pointer text-sm transition hover:bg-primary-dark"
                                                >
                                                    { "Close" }
                                                </button>
                                            </div>

                                            // Platform info section
                                            <div class="mb-6 pb-6 border-b border-gray-100">
                                                <h4 class="text-gray-700 mb-4">{ "Platform" }</h4>
                                                {
                                                    if let Some(p) = platform {
                                                        html! {
                                                            <>
                                                                <div class="mb-4">
                                                                    <strong class="inline-block min-w-[100px] text-gray-600">{ "Name:" }</strong>
                                                                    <span>{ &p.name }</span>
                                                                </div>
                                                                <div class="mb-4">
                                                                    <strong class="inline-block min-w-[100px] text-gray-600">{ "Type:" }</strong>
                                                                    <span>{ &p.platform_type }</span>
                                                                </div>
                                                                <div class="mb-4">
                                                                    <strong class="inline-block min-w-[100px] text-gray-600">{ "Base URL:" }</strong>
                                                                    <span>{ &p.base_url }</span>
                                                                </div>
                                                            </>
                                                        }
                                                    } else {
                                                        html! {
                                                            <div class="mb-4">
                                                                <strong class="inline-block min-w-[100px] text-gray-600">{ "Platform ID:" }</strong>
                                                                <span>{ log.llm_platform_id }</span>
                                                            </div>
                                                        }
                                                    }
                                                }
                                            </div>

                                            <div class="mb-6 pb-6 border-b border-gray-100">
                                                <div class="flex justify-between items-center mb-4">
                                                    <h4 class="text-gray-700">{ "Incoming Request (from user)" }</h4>
                                                    <button
                                                        class="bg-primary text-white px-3 py-2 text-xs ml-2 border-none rounded cursor-pointer transition hover:bg-primary-dark"
                                                        onclick={on_copy_incoming}
                                                    >
                                                        { "Copy as curl" }
                                                    </button>
                                                </div>
                                                <div class="mb-4">
                                                    <strong class="inline-block min-w-[100px] text-gray-600">{ "Method:" }</strong>
                                                    <span>{ &log.method }</span>
                                                </div>
                                                <div class="mb-4">
                                                    <strong class="inline-block min-w-[100px] text-gray-600">{ "Path:" }</strong>
                                                    <span>{ &log.path }</span>
                                                </div>
                                                {
                                                    if let Some(body) = &log.request_body {
                                                        let pretty_body = prettify_json(body);
                                                        html! {
                                                            <div class="mb-4">
                                                                <strong class="inline-block min-w-[100px] text-gray-600">{ "Body:" }</strong>
                                                                <pre class="mt-2 p-4 bg-gray-50 rounded overflow-x-auto text-xs whitespace-pre-wrap font-mono leading-normal">
                                                                    { pretty_body }
                                                                </pre>
                                                            </div>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                            </div>

                                            // Outgoing request section
                                            <div class="mb-6 pb-6 border-b border-gray-100">
                                                <div class="flex justify-between items-center mb-4">
                                                    <h4 class="text-gray-700">{ "Outgoing Request (to LLM platform)" }</h4>
                                                    {
                                                        if outgoing_curl.is_some() {
                                                            html! {
                                                                <button
                                                                    class="bg-primary text-white px-3 py-2 text-xs ml-2 border-none rounded cursor-pointer transition hover:bg-primary-dark"
                                                                    onclick={on_copy_outgoing}
                                                                >
                                                                    { "Copy as curl" }
                                                                </button>
                                                            }
                                                        } else {
                                                            html! {}
                                                        }
                                                    }
                                                </div>
                                                {
                                                    if let Some(url) = &log.outgoing_url {
                                                        html! {
                                                            <div class="mb-4">
                                                                <strong class="inline-block min-w-[100px] text-gray-600">{ "URL:" }</strong>
                                                                <span>{ url }</span>
                                                            </div>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                                {
                                                    if let Some(headers) = &log.outgoing_headers {
                                                        html! {
                                                            <div class="mb-4">
                                                                <strong class="inline-block min-w-[100px] text-gray-600">{ "Headers:" }</strong>
                                                                <pre class="mt-2 p-4 bg-gray-50 rounded overflow-x-auto text-xs">
                                                                    { headers }
                                                                </pre>
                                                            </div>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                                {
                                                    if let Some(body) = &log.outgoing_body {
                                                        let pretty_body = prettify_json(body);
                                                        html! {
                                                            <div class="mb-4">
                                                                <strong class="inline-block min-w-[100px] text-gray-600">{ "Body:" }</strong>
                                                                <pre class="mt-2 p-4 bg-gray-50 rounded overflow-x-auto text-xs whitespace-pre-wrap font-mono leading-normal">
                                                                    { pretty_body }
                                                                </pre>
                                                            </div>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                            </div>

                                            <div class="mb-6 pb-6 border-b-0">
                                                <h4 class="text-gray-700 mb-4">{ "Response (from LLM platform)" }</h4>
                                                {
                                                    if let Some(status) = log.response_status {
                                                        html! {
                                                            <div class="mb-4">
                                                                <strong class="inline-block min-w-[100px] text-gray-600">{ "Status:" }</strong>
                                                                <span>{ status }</span>
                                                            </div>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                                {
                                                    if let Some(body) = &log.response_body {
                                                        let pretty_body = prettify_json(body);
                                                        html! {
                                                            <div class="mb-4">
                                                                <strong class="inline-block min-w-[100px] text-gray-600">{ "Body:" }</strong>
                                                                <pre class="mt-2 p-4 bg-gray-50 rounded overflow-x-auto text-xs whitespace-pre-wrap font-mono leading-normal">
                                                                    { pretty_body }
                                                                </pre>
                                                            </div>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                                {
                                                    if let Some(error) = &log.error {
                                                        html! {
                                                            <div class="mb-4 text-danger">
                                                                <strong class="inline-block min-w-[100px]">{ "Error:" }</strong>
                                                                <span>{ error }</span>
                                                            </div>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                            </div>
                                        </div>
                                    }
                                } else {
                                    html! {
                                        <div class="bg-white rounded-lg p-6 shadow-md max-h-[80vh] overflow-y-auto flex items-center justify-center text-gray-500">
                                            <p>{ "Select a log to view details" }</p>
                                        </div>
                                    }
                                }
                            }
                        </div>
                    }
                }
            }
        </div>
    }
}
