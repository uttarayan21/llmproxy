#!/bin/bash
# Development setup script for LLMPROXY

set -e

echo "🚀 Starting LLMPROXY development environment..."

# Check if trunk is installed
if ! command -v trunk &> /dev/null; then
    echo "❌ trunk is not installed. Please install it:"
    echo "   cargo install trunk"
    exit 1
fi

# Export development environment variable
export DEV_DEFAULT_USER="dev-user"
export DATABASE_URL="sqlite:llmproxy.db"
export HOST="127.0.0.1"
export PORT="8080"

# Start backend in background
echo "🔧 Starting backend on http://127.0.0.1:8080..."
cargo run --package backend &
BACKEND_PID=$!

# Wait for backend to start
echo "⏳ Waiting for backend to initialize..."
sleep 3

# Start frontend
echo "🎨 Starting frontend on http://127.0.0.1:8081..."
echo ""
echo "📝 Access the application at: http://127.0.0.1:8081"
echo "👤 Default dev user: dev-user"
echo ""
cd frontend
trunk build

cd ..

caddy dev --config Caddyfile

# Cleanup on exit
trap "echo '🛑 Stopping backend...'; kill $BACKEND_PID 2>/dev/null" EXIT
