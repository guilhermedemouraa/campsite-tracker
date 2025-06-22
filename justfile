# Campsite Tracker Development Commands

@_default:
  just --list

localusers:
  curl -X GET http://localhost:8080/api/auth/users

awsusers:
  curl -X GET http://18.144.164.38:8080/api/auth/users
# === Backend Commands ===

# Format backend code
format-backend:
  #!/usr/bin/env bash
  cd backend
  echo "🦀 Formatting Rust code..."
  cargo +nightly fmt
  echo "📝 Formatting TOML files..."
  taplo fmt

# Lint backend code
lint-backend:
  #!/usr/bin/env bash
  cd backend
  echo "🔍 Running Clippy..."
  cargo clippy -- -D warnings
  echo "✅ Backend linting complete"

# Run backend tests
test-backend:
  #!/usr/bin/env bash
  cd backend
  echo "🧪 Running backend tests..."
  cargo test

# Build backend for development
build-backend:
  #!/usr/bin/env bash
  cd backend
  echo "🔨 Building backend..."
  cargo build

# Run backend locally
run-backend:
  #!/usr/bin/env bash
  cd backend
  echo "🚀 Starting backend server..."
  cargo run

# === Frontend Commands ===

# Install frontend dependencies
install-frontend:
  #!/usr/bin/env bash
  cd frontend
  echo "📦 Installing frontend dependencies..."
  npm install

# Format frontend code
format-frontend:
  #!/usr/bin/env bash
  cd frontend
  echo "💅 Formatting frontend code..."
  npx prettier --write "src/**/*.{ts,tsx,js,jsx,css}"

# Lint frontend code
lint-frontend:
  #!/usr/bin/env bash
  cd frontend
  echo "🔍 Linting frontend code..."
  npx eslint src --ext .ts,.tsx,.js,.jsx --fix
  echo "🎯 Type checking..."
  npx tsc --noEmit

# Run frontend tests
test-frontend:
  #!/usr/bin/env bash
  cd frontend
  echo "🧪 Running frontend tests..."
  npm test -- --watchAll=false

# Build frontend for production
build-frontend:
  #!/usr/bin/env bash
  cd frontend
  echo "🏗️ Building frontend..."
  npm run build

# Run frontend dev server
dev-frontend:
  #!/usr/bin/env bash
  cd frontend
  echo "🚀 Starting frontend dev server..."
  npm start

# === Deploy Commands ===

# Format deployment scripts
format-deploy:
  #!/usr/bin/env bash
  cd deploy
  echo "📜 Formatting shell scripts..."
  for file in *.sh; do
    if [ -f "$file" ]; then
      echo "Formatting $file..."
      # Use shfmt if available, otherwise just check syntax
      if command -v shfmt >/dev/null 2>&1; then
        shfmt -w -i 4 -ci "$file"
      else
        bash -n "$file" && echo "✅ $file syntax OK"
      fi
    fi
  done

# Lint deployment scripts  
lint-deploy:
  #!/usr/bin/env bash
  cd deploy
  echo "🔍 Linting shell scripts..."
  for file in *.sh; do
    if [ -f "$file" ]; then
      echo "Checking $file..."
      if command -v shellcheck >/dev/null 2>&1; then
        shellcheck "$file"
      else
        bash -n "$file" && echo "✅ $file syntax OK"
      fi
    fi
  done

# Make deployment scripts executable
chmod-deploy:
  #!/usr/bin/env bash
  cd deploy
  echo "🔐 Making deployment scripts executable..."
  chmod +x *.sh
  ls -la *.sh

# === Docker Commands ===

# Format Dockerfile
format-docker:
  #!/usr/bin/env bash
  echo "🐳 Checking Dockerfile..."
  if command -v hadolint >/dev/null 2>&1; then
    hadolint Dockerfile
  else
    echo "📝 Dockerfile exists: $(test -f Dockerfile && echo '✅' || echo '❌')"
  fi

# === Combined Commands ===

# Format all code
format: format-backend format-frontend format-deploy format-docker
  echo "✨ All code formatted!"

# Lint all code
lint: lint-backend lint-frontend lint-deploy
  echo "🔍 All code linted!"

# Test all code
test: test-backend test-frontend
  echo "🧪 All tests complete!"

# Build everything
build: build-backend build-frontend
  echo "🏗️ Full build complete!"

# === Development Workflow ===

# Setup development environment
setup:
  #!/usr/bin/env bash
  echo "🛠️ Setting up development environment..."
  
  # Install Rust nightly for formatting
  echo "Installing Rust nightly..."
  rustup toolchain install nightly
  rustup component add rustfmt --toolchain nightly
  
  # Install frontend deps
  just install-frontend
  
  # Make deploy scripts executable
  just chmod-deploy
  
  echo "✅ Development environment ready!"

# Pre-commit checks (run before committing)
check: format lint test
  echo "✅ Pre-commit checks passed!"

# Quick development iteration
dev:
  #!/usr/bin/env bash
  echo "🚀 Starting development servers..."
  
  # Start backend in background
  cd backend && cargo run &
  BACKEND_PID=$!
  
  # Start frontend in background  
  cd frontend && npm start &
  FRONTEND_PID=$!
  
  echo "Backend PID: $BACKEND_PID"
  echo "Frontend PID: $FRONTEND_PID"
  echo "Press Ctrl+C to stop both servers"
  
  # Wait for interrupt
  trap 'kill $BACKEND_PID $FRONTEND_PID 2>/dev/null' INT
  wait

# === Deployment ===

# Deploy to AWS
deploy:
  #!/usr/bin/env bash
  cd deploy
  echo "🚀 Deploying to AWS..."
  ./build_and_deploy.sh

# Check deployment status
status:
  #!/usr/bin/env bash
  cd deploy
  source .env
  INSTANCE_IP=$(aws ec2 describe-instances \
    --instance-ids $INSTANCE_ID \
    --query 'Reservations[0].Instances[0].PublicIpAddress' \
    --output text)
  
  echo "🌐 App URL: http://$INSTANCE_IP:8080"
  echo "📋 Container status:"
  ssh -i campsite-key.pem -o StrictHostKeyChecking=no ec2-user@$INSTANCE_IP \
    'sudo docker ps | grep campsite'

# View deployment logs
logs:
  #!/usr/bin/env bash
  cd deploy
  source .env
  INSTANCE_IP=$(aws ec2 describe-instances \
    --instance-ids $INSTANCE_ID \
    --query 'Reservations[0].Instances[0].PublicIpAddress' \
    --output text)
  
  echo "📋 Recent logs:"
  ssh -i campsite-key.pem ec2-user@$INSTANCE_IP 'sudo docker logs -f campsite-tracker'

start_instance:
  #!/usr/bin/env bash
  cd deploy
  source .env
  echo "🚀 Starting EC2 instance..."
  aws ec2 start-instances --instance-ids $INSTANCE_ID
  echo "Instance starting..."

stop_instance:
  #!/usr/bin/env bash
  cd deploy
  source .env
  echo "🛑 Stopping EC2 instance..."
  aws ec2 stop-instances --instance-ids $INSTANCE_ID
  echo "Instance stopping..."

# === Cleanup ===

# Clean build artifacts
clean:
  #!/usr/bin/env bash
  echo "🧹 Cleaning build artifacts..."
  cd backend && cargo clean
  cd ../frontend && rm -rf build node_modules/.cache
  echo "✅ Cleanup complete!"