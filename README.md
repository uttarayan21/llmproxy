# LLMPROXY

# SLOP: Slop warning this full project was vibecoded with opencode + claude-sonnet-4.5

A simple observability platform for LLM APIs built with Rust.

## Features

1. **Proxy Layer** - Acts as a proxy between your application and any LLM platform (OpenAI, self-hosted Ollama, etc.)
2. **Observability** - Log and observe all API calls that your app makes to LLM platforms
3. **Multi-Platform Support** - Configure multiple LLM platforms with different API keys
4. **Per-Platform API Keys** - Each API key is tied to a specific platform, simplifying client configuration
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
- [just](https://github.com/casey/just) command runner (recommended)
- Trunk (for building frontend)
- Caddy (optional, for development with authentication)

### Quick Start

The easiest way to build and run the project is using `just`:

```bash
# Install just if you don't have it
cargo install just

# Check dependencies
just check-deps

# Build the project
just build

# Run the backend server
just run

# Or run in development mode with auto-reload
just dev

# For development with Caddy reverse proxy (adds auth header):
# Terminal 1:
just serve-backend

# Terminal 2:
just serve-caddy
# Access at http://localhost:8081 with automatic Remote-User header
```

See all available commands with:
```bash
just --list
```

### Manual Installation

If you prefer not to use `just`:

1. Clone the repository:
```bash
git clone <repository-url>
cd llmproxy
```

2. Configure the application (optional):

Create a `config.toml` file (see [Configuration](#configuration) section below) or use environment variables:

```bash
export DATABASE_URL=sqlite:llmproxy.db
export HOST=0.0.0.0
export PORT=8080
export AUTH_ENABLE_SESSION=true
export AUTH_ENABLE_HEADER=true
export AUTH_HEADER_NAME=Remote-User
```

3. Build and run the backend:
```bash
cd backend
cargo run --release
```

The server will start on `http://localhost:8080`

## Configuration

LLMPROXY can be configured using a `config.toml` file or environment variables. Configuration files are loaded from `config.toml` in the current directory, or from a custom path via the `CONFIG_PATH` environment variable.

### Configuration File

Create a `config.toml` file based on `config.example.toml`:

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "sqlite:llmproxy.db"

[auth]
# Enable/disable individual authentication methods
# Multiple methods can be enabled simultaneously

enable_session = true    # Session-based auth (login/register)
enable_header = true     # Reverse proxy header auth
header_name = "Remote-User"
enable_basic = false     # HTTP Basic Authentication
disable_all = false      # Disable all auth (dangerous!)
```

### Environment Variables

All configuration options can be overridden with environment variables:

- `CONFIG_PATH` - Path to config file (default: `config.toml`)
- `HOST` - Server host (default: `0.0.0.0`)
- `PORT` - Server port (default: `8080`)
- `DATABASE_URL` - Database connection string (default: `sqlite:llmproxy.db`)
- `AUTH_ENABLE_SESSION` - Enable session-based authentication (default: `true`)
- `AUTH_ENABLE_HEADER` - Enable header-based authentication (default: `true`)
- `AUTH_HEADER_NAME` - Header name for proxy auth (default: `Remote-User`)
- `AUTH_ENABLE_BASIC` - Enable HTTP Basic Authentication (default: `false`)
- `AUTH_DISABLE_ALL` - Disable all authentication (default: `false`, **USE WITH CAUTION!**)

### Authentication Methods

LLMPROXY supports multiple authentication methods that can be enabled independently:

1. **Session Authentication** (`enable_session`)
   - Users can register and login via the web UI
   - Passwords are hashed using industry-standard password-auth
   - Sessions persist for 7 days

2. **Header Authentication** (`enable_header`)
   - Authenticates users via a reverse proxy header (e.g., from Caddy, nginx)
   - Automatically creates users on first access
   - Configurable header name (default: `Remote-User`)

3. **HTTP Basic Authentication** (`enable_basic`)
   - Standard HTTP Basic Authentication
   - Uses the same user accounts as session auth
   - Useful for API clients and scripts

4. **Disable All** (`disable_all`)
   - **WARNING**: Completely disables authentication
   - All requests are allowed and attributed to an "anonymous" user
   - Only use in trusted, isolated environments

**Recommended configurations:**
- **Production with reverse proxy**: `enable_header=true`, others `false`
- **Development**: `enable_session=true` and `enable_header=true`
- **Standalone**: `enable_session=true`, others `false`
- **API-only**: `enable_basic=true`, others `false`


### Building the Frontend

The frontend is automatically built and embedded into the backend binary. For manual frontend development:

```bash
# Install trunk
cargo install trunk

# Build and serve the frontend (in development)
cd frontend
trunk serve --port 8082

# Or build for production
trunk build --release
```

### Production Deployment

#### NixOS (Recommended)

LLMPROXY includes a complete NixOS module for easy deployment:

```nix
# In your configuration.nix
{
  imports = [ ./path/to/llmproxy/nixos-module.nix ];

  services.llmproxy = {
    enable = true;
    nginx = {
      enable = true;
      domain = "llmproxy.example.com";
      authMethod = "basic";
      basicAuthFile = "/etc/nginx/.htpasswd";
    };
  };
}
```

See [NIXOS_DEPLOYMENT.md](./NIXOS_DEPLOYMENT.md) for complete documentation.

#### Manual Deployment

For other systems, you'll need a reverse proxy (like nginx or Caddy) to:
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

Navigate to "API Keys" and generate a key for your application. When creating an API key, you'll need to:

1. Give it a descriptive name
2. **Select the LLM platform** it should route to

Each API key is tied to a specific platform, so you don't need to specify the platform in your application's URL. Copy the key immediately as it won't be shown again.

### 4. Configure Your Application

Point your application to use the proxy. The API key automatically determines which platform to route to:

```python
import openai

# Configure to use the proxy - no platform ID needed in URL!
client = openai.OpenAI(
    base_url="http://localhost:8080/proxy",
    api_key="llmp_xxxxxxxxxxxxx"  # Your proxy API key (identifies both auth and platform)
)

response = client.chat.completions.create(
    model="gpt-4",
    messages=[{"role": "user", "content": "Hello!"}]
)
```

The proxy will:
1. Authenticate your request using the proxy API key
2. **Automatically determine which platform to route to** from the API key
3. Forward the request to the configured platform with its API key
4. Log the request and response
5. Return the response to your app

**Note**: Your application code doesn't need to know about platform IDs. The API key handles both authentication and platform routing.

### 5. View Request Logs

Navigate to "Request Logs" in the dashboard to see all logged requests, including:
- Request method, path, headers, and body
- Response status, headers, and body
- Duration in milliseconds
- Timestamps

## API Endpoints

Complete API documentation is available in OpenAPI 3.0 format:

- **Full Documentation**: See [API_DOCUMENTATION.md](./API_DOCUMENTATION.md)
- **OpenAPI Spec**: [openapi.yaml](./openapi.yaml) or [openapi.json](./openapi.json)
- **Interactive Docs**: Import the spec into [Swagger Editor](https://editor.swagger.io/) or [Redoc](https://redocly.com/redoc/)

### Quick Reference

**Management API** (requires Remote-User header):

- `GET /api/user` - Get current authenticated user
- `GET /api/platforms` - List all LLM platforms
- `POST /api/platforms` - Create new LLM platform
- `DELETE /api/platforms/:id` - Delete LLM platform
- `GET /api/keys` - List all proxy API keys
- `POST /api/keys` - Generate new proxy API key
- `DELETE /api/keys/:id` - Revoke proxy API key
- `GET /api/logs?limit=100` - List request logs
- `GET /api/logs/:id` - Get specific log details

**Proxy Endpoint** (requires Authorization: Bearer header with proxy API key):

- `ANY /proxy/*path` - Proxy requests to LLM platform (platform determined by API key)

## Database Schema

The SQLite database contains four main tables:

- `users` - Auto-created from Remote-User header
- `llm_platforms` - LLM platform configurations (with API keys)
- `proxy_api_keys` - Generated API keys for proxy access (each linked to a specific platform)
- `request_logs` - All proxied requests and responses with timing data

## Development

This project uses [just](https://github.com/casey/just) as a command runner for common development tasks.

### Common Commands

```bash
# Show all available commands
just --list

# Build and run
just build          # Build release binary
just build-dev      # Build debug binary
just run            # Run backend (release)
just dev            # Run with auto-reload (requires cargo-watch)

# Development with Caddy
just serve-backend  # Start backend in background
just serve-caddy    # Start Caddy reverse proxy with auth
just stop           # Stop all services

# Testing and quality
just test           # Run tests
just test-verbose   # Run tests with output
just lint           # Run clippy
just fix            # Auto-fix clippy issues
just fmt            # Format code

# Database management
just db-setup       # Setup database
just db-backup      # Backup database
just db-reset       # Reset database (WARNING: deletes data)
just db-schema      # View schema
just db-cli         # Open SQLite CLI

# Utilities
just clean          # Clean build artifacts
just rebuild        # Clean and rebuild
just check-deps     # Check installed dependencies
just install-deps   # Install dev dependencies
just info           # Show project info
```

### Manual Development Commands

If not using `just`:

#### Run tests

```bash
cargo test
```

#### Format code

```bash
cargo fmt
```

#### Run linter

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
4. **Key Generation**: User generates proxy API keys via UI, selecting which platform each key routes to
5. **Proxy Request**: App → Proxy (with proxy API key) → LLM Platform (determined by API key)
6. **Logging**: All requests/responses logged to database
7. **Observability**: View logs in real-time via UI

## License

MIT

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.
