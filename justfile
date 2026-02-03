# LLMPROXY - Just commands
# Run `just` or `just --list` to see all available commands

# Default recipe - show available commands
default:
    @just --list

# Environment variables for development
# export DEV_DEFAULT_USER := "dev-user"
export DATABASE_URL := "sqlite:llmproxy.db"
export HOST := "127.0.0.1"
export PORT := "8080"

# Build the entire project (backend + frontend)
build:
    @echo "🔨 Building backend..."
    cd backend && cargo build --release
    @echo "🎨 Building frontend with Trunk..."
    cd frontend && trunk build --release
    @echo "✅ Build complete!"

# Build for development (debug mode)
build-dev:
    @echo "🔨 Building backend (debug)..."
    cd backend && cargo build
    @echo "✅ Build complete!"

# Run the backend server (production mode)
run: build
    @echo "🚀 Starting LLMPROXY backend..."
    @echo "📝 Access at: http://127.0.0.1:8080"
    cargo run --release

# Run the backend server (development mode with auto-reload)
dev:
    @echo "🚀 Starting LLMPROXY in development mode..."
    @echo ""
    @echo "Backend will run on: http://127.0.0.1:8080"
    @echo "Run 'just serve-caddy' in another terminal for auth proxy on :8081"
    @echo "👤 Default dev user: dev-user"
    @echo ""
    cd backend && cargo watch -x run

# Run backend with Caddy reverse proxy for auth
serve: serve-backend serve-caddy

# Start backend server in background
serve-backend:
    #!/usr/bin/env bash
    set -e
    echo "🔧 Starting backend on http://127.0.0.1:8080..."
    cd backend && cargo run --release &
    echo $! > .backend.pid
    echo "✅ Backend started (PID: $(cat .backend.pid))"
    echo "⏳ Waiting for backend to initialize..."
    sleep 3

# Start Caddy reverse proxy
serve-caddy:
    @echo "🎨 Starting Caddy reverse proxy on http://127.0.0.1:8081..."
    @echo ""
    @echo "📝 Access the application at: http://127.0.0.1:8081"
    @echo "👤 Default dev user: dev-user"
    @echo ""
    caddy run

# Stop all running services
stop:
    #!/usr/bin/env bash
    if [ -f .backend.pid ]; then
        PID=$(cat .backend.pid)
        echo "🛑 Stopping backend (PID: $PID)..."
        kill $PID 2>/dev/null || echo "Backend already stopped"
        rm .backend.pid
    else
        echo "No backend PID file found"
    fi
    
    # Stop Caddy if running
    pkill caddy 2>/dev/null && echo "🛑 Stopped Caddy" || echo "Caddy not running"

# Build frontend only
build-frontend:
    @echo "🎨 Building frontend with Trunk..."
    cd frontend && trunk build --release
    @echo "✅ Frontend built to frontend/dist/"

# Build frontend in development mode
build-frontend-dev:
    @echo "🎨 Building frontend (debug)..."
    cd frontend && trunk build
    @echo "✅ Frontend built to frontend/dist/"

# Watch and rebuild frontend on changes
watch-frontend:
    @echo "👀 Watching frontend for changes..."
    cd frontend && trunk serve --port 8082

# Run tests
test:
    @echo "🧪 Running backend tests..."
    cd backend && cargo test

# Run tests with output
test-verbose:
    @echo "🧪 Running backend tests (verbose)..."
    cd backend && cargo test -- --nocapture

# Format all code
fmt:
    @echo "✨ Formatting code..."
    cd backend && cargo fmt
    cd frontend && cargo fmt
    @echo "✅ Code formatted!"

# Run linter (clippy)
lint:
    @echo "🔍 Running clippy..."
    cd backend && cargo clippy -- -D warnings
    cd frontend && cargo clippy -- -D warnings

# Fix linter warnings automatically
fix:
    @echo "🔧 Auto-fixing linter issues..."
    cd backend && cargo clippy --fix --allow-dirty --allow-staged
    cd frontend && cargo clippy --fix --allow-dirty --allow-staged

# Check code without building
check:
    @echo "🔍 Checking backend..."
    cd backend && cargo check
    @echo "🔍 Checking frontend..."
    cd frontend && cargo check

# Clean build artifacts
clean:
    @echo "🧹 Cleaning build artifacts..."
    cd backend && cargo clean
    cd frontend && cargo clean
    rm -rf frontend/dist
    @echo "✅ Clean complete!"

# Setup database (run migrations)
db-setup:
    #!/usr/bin/env bash
    set -e
    echo "📦 Setting up database..."
    for migration in backend/migrations/*.sql; do
        echo "  Running: $(basename $migration)"
        sqlite3 llmproxy.db < "$migration"
    done
    echo "✅ Database setup complete!"

# Backup database
db-backup:
    @echo "💾 Backing up database..."
    cp llmproxy.db llmproxy.db.backup-$(date +%Y%m%d-%H%M%S)
    @echo "✅ Database backed up!"

# Reset database (WARNING: deletes all data)
db-reset:
    @echo "⚠️  Resetting database (all data will be lost)..."
    rm -f llmproxy.db
    just db-setup
    @echo "✅ Database reset complete!"

# View database schema
db-schema:
    @echo "📊 Database schema:"
    sqlite3 llmproxy.db ".schema"

# Open database CLI
db-cli:
    @echo "💻 Opening database CLI (type .quit to exit)..."
    sqlite3 llmproxy.db

# Install development dependencies
install-deps:
    @echo "📦 Installing development dependencies..."
    cargo install trunk
    cargo install cargo-watch
    cargo install sqlx-cli --no-default-features --features sqlite
    @echo "✅ Dependencies installed!"

# Check if all required tools are installed
check-deps:
    #!/usr/bin/env bash
    echo "🔍 Checking dependencies..."
    
    command -v cargo >/dev/null 2>&1 || { echo "❌ cargo not found"; exit 1; }
    echo "✅ cargo found"
    
    command -v trunk >/dev/null 2>&1 || { echo "⚠️  trunk not found (run: cargo install trunk)"; }
    echo "✅ trunk found" 2>/dev/null || true
    
    command -v caddy >/dev/null 2>&1 || { echo "⚠️  caddy not found (optional for dev auth)"; }
    echo "✅ caddy found" 2>/dev/null || true
    
    command -v sqlite3 >/dev/null 2>&1 || { echo "⚠️  sqlite3 not found"; }
    echo "✅ sqlite3 found" 2>/dev/null || true
    
    command -v cargo-watch >/dev/null 2>&1 || { echo "⚠️  cargo-watch not found (run: cargo install cargo-watch)"; }
    echo "✅ cargo-watch found" 2>/dev/null || true
    
    echo ""
    echo "✅ Dependency check complete!"

# Update all dependencies
update-deps:
    @echo "🔄 Updating dependencies..."
    cd backend && cargo update
    cd frontend && cargo update
    @echo "✅ Dependencies updated!"

# Show project info
info:
    @echo "📊 LLMPROXY Project Information"
    @echo ""
    @echo "Backend:"
    @cd backend && cargo --version
    @echo ""
    @echo "Frontend:"
    @cd frontend && cargo --version
    @echo ""
    @echo "Database: SQLite"
    @test -f llmproxy.db && echo "Database exists: llmproxy.db" || echo "Database not initialized (run: just db-setup)"
    @echo ""
    @echo "Environment:"
    @echo "  DATABASE_URL: $DATABASE_URL"
    @echo "  HOST: $HOST"
    @echo "  PORT: $PORT"
    @echo "  DEV_DEFAULT_USER: $DEV_DEFAULT_USER"

# View recent logs
logs:
    @echo "📋 Recent logs:"
    @test -f llmproxy.log && tail -n 50 llmproxy.log || echo "No log file found"

# Full clean and rebuild
rebuild: clean build
    @echo "🎉 Rebuild complete!"

# Quick development start (builds and runs with Caddy)
start: build serve-backend serve-caddy

# Production build
release: clean build db-backup
    @echo "🎉 Production build complete!"
    @echo "Binary location: backend/target/release/backend"
