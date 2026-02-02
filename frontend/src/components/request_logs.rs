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

#[function_component(RequestLogs)]
pub fn request_logs() -> Html {
    let logs = use_state(Vec::<RequestLog>::new);
    let selected_log = use_state(|| Option::<RequestLog>::None);
    let loading = use_state(|| true);

    {
        let logs = logs.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(response) = Request::get("/api/logs?limit=50").send().await
                    && let Ok(data) = response.json::<Vec<RequestLog>>().await {
                        logs.set(data);
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
                                    html! {
                                        <div class="log-detail">
                                            <div class="detail-header">
                                                <h3>{ "Request Details" }</h3>
                                                <button onclick={on_close_detail}>{ "Close" }</button>
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
