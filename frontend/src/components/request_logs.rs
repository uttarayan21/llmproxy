use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
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
                    && let Ok(data) = response.json::<Vec<RequestLog>>().await {
                        logs.set(data);
                    }
                // Fetch platforms
                if let Ok(response) = Request::get("/api/platforms").send().await
                    && let Ok(data) = response.json::<Vec<LlmPlatform>>().await {
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
        <div class="request-logs">
            <h2>{ "Request Logs" }</h2>

            {
                if *loading {
                    html! { <div class="loading">{ "Loading logs..." }</div> }
                } else if logs.is_empty() {
                    html! { <div class="empty">{ "No request logs yet" }</div> }
                } else {
                    html! {
                        <div class="logs-container">
                            <div class="logs-list">
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
                                                class="log-item"
                                                onclick={Callback::from(move |_| on_select.emit(log_clone.clone()))}
                                            >
                                                <div class="log-header">
                                                    <span class="method">{ &log.method }</span>
                                                    <span class="path">{ &log.path }</span>
                                                    {
                                                        if let Some(status) = log.response_status {
                                                            html! { <span class={format!("status status-{}", status / 100)}>{ status }</span> }
                                                        } else {
                                                            html! { <span class="status status-error">{ "Error" }</span> }
                                                        }
                                                    }
                                                </div>
                                                <div class="log-meta">
                                                    <span class="platform">{ format!("Platform: {}", platform_name) }</span>
                                                    {
                                                        if let Some(duration) = log.duration_ms {
                                                            html! { <span class="duration">{ format!("{}ms", duration) }</span> }
                                                        } else {
                                                            html! {}
                                                        }
                                                    }
                                                    <span class="timestamp">{ &log.created_at }</span>
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
                                    
                                    html! {
                                        <div class="log-detail">
                                            <div class="detail-header">
                                                <h3>{ "Request Details" }</h3>
                                                <button onclick={on_close_detail}>{ "Close" }</button>
                                            </div>

                                            // Platform info section
                                            <div class="detail-section platform-section">
                                                <h4>{ "Platform" }</h4>
                                                {
                                                    if let Some(p) = platform {
                                                        html! {
                                                            <>
                                                                <div class="detail-item">
                                                                    <strong>{ "Name:" }</strong>
                                                                    <span>{ &p.name }</span>
                                                                </div>
                                                                <div class="detail-item">
                                                                    <strong>{ "Type:" }</strong>
                                                                    <span>{ &p.platform_type }</span>
                                                                </div>
                                                                <div class="detail-item">
                                                                    <strong>{ "Base URL:" }</strong>
                                                                    <span>{ &p.base_url }</span>
                                                                </div>
                                                            </>
                                                        }
                                                    } else {
                                                        html! {
                                                            <div class="detail-item">
                                                                <strong>{ "Platform ID:" }</strong>
                                                                <span>{ log.llm_platform_id }</span>
                                                            </div>
                                                        }
                                                    }
                                                }
                                            </div>

                                            <div class="detail-section">
                                                <h4>{ "Request" }</h4>
                                                <div class="detail-item">
                                                    <strong>{ "Method:" }</strong>
                                                    <span>{ &log.method }</span>
                                                </div>
                                                <div class="detail-item">
                                                    <strong>{ "Path:" }</strong>
                                                    <span>{ &log.path }</span>
                                                </div>
                                                {
                                                    if let Some(body) = &log.request_body {
                                                        html! {
                                                            <div class="detail-item">
                                                                <strong>{ "Body:" }</strong>
                                                                <pre>{ body }</pre>
                                                            </div>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                            </div>

                                            <div class="detail-section">
                                                <h4>{ "Response" }</h4>
                                                {
                                                    if let Some(status) = log.response_status {
                                                        html! {
                                                            <div class="detail-item">
                                                                <strong>{ "Status:" }</strong>
                                                                <span>{ status }</span>
                                                            </div>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                                {
                                                    if let Some(body) = &log.response_body {
                                                        html! {
                                                            <div class="detail-item">
                                                                <strong>{ "Body:" }</strong>
                                                                <pre>{ body }</pre>
                                                            </div>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                                {
                                                    if let Some(error) = &log.error {
                                                        html! {
                                                            <div class="detail-item error">
                                                                <strong>{ "Error:" }</strong>
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
                                        <div class="log-detail-placeholder">
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
