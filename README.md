# LLMPROXY

A simple observability platform for LLM APIs built with Rust.

## Features

1. **Proxy Layer** - Acts as a proxy between your application and any LLM platform (OpenAI, self-hosted Ollama, etc.)
2. **Observability** - Log and observe all API calls that your app makes to LLM platforms
3. **Multi-Platform Support** - Configure multiple LLM platforms with different API keys
4. **API Key Management** - Secure proxy access with generated API keys
5. **Web Dashboard** - Clean UI to view request logs and manage platforms/keys
6. **Reverse Proxy Auth** - Auto-create users based on Remote-User header for frontend authentication

## Tech Stack

- **Backend**: Rust 2024 edition
- **Web Framework**: Axum
- **HTTP Client**: Reqwest
- **Async Runtime**: Tokio
- **Database**: SQLite (via SQLx - easily switchable to other databases)
- **Frontend**: Yew (Rust WebAssembly framework)
- **Authentication**: Reverse proxy header auth for UI, API key auth for proxy requests

## Getting Started

### Prerequisites

- Rust 2024 edition (nightly or stable with edition = "2024")
- Cargo
- Trunk (for building frontend)

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd llmproxy
```

2. Set environment variables (optional):
```bash
export DATABASE_URL=sqlite:llmproxy.db
export HOST=0.0.0.0
export PORT=8080
```

3. Build and run the backend:
```bash
cargo run --package backend
```

The server will start on `http://localhost:8080`

### Building the Frontend

The frontend is built with Yew and requires `trunk`:

```bash
# Install trunk
cargo install trunk

# Build and serve the frontend (in development)
cd frontend
trunk serve --port 8081

# Or build for production
trunk build --release
```

### Production Deployment

For production, you'll need a reverse proxy (like nginx or Caddy) to:
1. Serve the frontend static files
2. Proxy `/api/*` and `/proxy/*` requests to the backend
3. Add the `Remote-User` header for authentication

## Usage

### 1. Access the Dashboard

Access the web UI at `http://localhost:8080` (or your configured port). The UI uses reverse proxy authentication via the `Remote-User` header.

For development without a reverse proxy, you can modify nginx or use a tool to add the header:

```bash
# Example nginx config snippet
location / {
    proxy_pass http://localhost:8080;
    proxy_set_header Remote-User "testuser";
}
```

### 2. Configure LLM Platforms

In the dashboard, navigate to "LLM Platforms" and add your platforms:

- **Name**: A friendly name (e.g., "OpenAI Production")
- **Base URL**: The API base URL (e.g., `https://api.openai.com/v1`)
- **API Key**: Your LLM provider's API key
- **Platform Type**: openai, ollama, or custom

### 3. Generate Proxy API Keys

Navigate to "API Keys" and generate a key for your application. Copy it immediately as it won't be shown again.

### 4. Configure Your Application

Point your application to use the proxy:

```python
import openai

# Configure to use the proxy
openai.api_base = "http://localhost:8080/proxy/1"  # 1 is your platform ID

# Use your proxy API key for authentication
client = openai.OpenAI(
    base_url="http://localhost:8080/proxy/1",
    api_key="llmp_xxxxxxxxxxxxx"  # Your proxy API key, not OpenAI key
)

response = client.chat.completions.create(
    model="gpt-4",
    messages=[{"role": "user", "content": "Hello!"}]
)
```

The proxy will:
1. Authenticate your request using the proxy API key
2. Forward the request to the configured platform with its API key
3. Log the request and response
4. Return the response to your app

### 5. View Request Logs

Navigate to "Request Logs" in the dashboard to see all logged requests, including:
- Request method, path, headers, and body
- Response status, headers, and body
- Duration in milliseconds
- Timestamps

## API Endpoints

### Management API (requires Remote-User header)

- `GET /api/user` - Get current authenticated user
- `GET /api/platforms` - List all LLM platforms
- `POST /api/platforms` - Create new LLM platform
- `DELETE /api/platforms/:id` - Delete LLM platform
- `GET /api/keys` - List all proxy API keys
- `POST /api/keys` - Generate new proxy API key
- `DELETE /api/keys/:id` - Revoke proxy API key
- `GET /api/logs?limit=100` - List request logs
- `GET /api/logs/:id` - Get specific log details

### Proxy Endpoint (requires Authorization: Bearer header with proxy API key)

- `ANY /proxy/:platform_id/*path` - Proxy requests to LLM platform

## Database Schema

The SQLite database contains four main tables:

- `users` - Auto-created from Remote-User header
- `llm_platforms` - LLM platform configurations (with API keys)
- `proxy_api_keys` - Generated API keys for proxy access
- `request_logs` - All proxied requests and responses with timing data

## Development

### Run tests

```bash
cargo test
```

### Format code

```bash
cargo fmt
```

### Run linter

```bash
cargo clippy
```

## Architecture

```
                                ┌──────────────────┐
                                │   Reverse Proxy  │
                                │  (adds Remote-   │
                                │   User header)   │
                                └────────┬─────────┘
                                         │
                                         ▼
┌─────────────┐         ┌───────────────────────┐         ┌─────────────┐
│             │  proxy  │                       │ forward │             │
│  Your App   ├────────►│     LLM Proxy         ├────────►│  LLM API    │
│             │ API key │  (Rust/Axum/SQLite)   │ + API   │  (OpenAI,   │
└─────────────┘         │                       │  key    │   Ollama)   │
                        └───────────┬───────────┘         └─────────────┘
                                    │
                                    │ logs
                                    ▼
                            ┌───────────────┐
                            │  SQLite DB    │
                            │  - Users      │
                            │  - Platforms  │
                            │  - API Keys   │
                            │  - Logs       │
                            └───────────────┘
```

### Flow:

1. **UI Access**: Browser → Reverse Proxy (adds Remote-User) → Frontend
2. **User Creation**: Automatic on first UI access based on Remote-User header
3. **Platform Setup**: User adds LLM platforms (with their API keys) via UI
4. **Key Generation**: User generates proxy API keys via UI
5. **Proxy Request**: App → Proxy (with proxy API key) → LLM Platform (with platform API key)
6. **Logging**: All requests/responses logged to database
7. **Observability**: View logs in real-time via UI

## License

MIT

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.
