#!/bin/bash
# Development setup script for LLMPROXY

set -e

echo "Starting LLMPROXY development environment..."

# Check if trunk is installed
if ! command -v trunk &> /dev/null; then
    echo "trunk is not installed. Installing..."
    cargo install trunk
fi

# Start backend in background
echo "Starting backend on port 8080..."
cd backend
cargo run &
BACKEND_PID=$!
cd ..

# Wait for backend to start
sleep 3

# Start frontend
echo "Starting frontend on port 8081..."
cd frontend
trunk serve --port 8081

# Cleanup on exit
trap "kill $BACKEND_PID 2>/dev/null" EXIT
