# Build frontend
FROM --platform=linux/amd64 node:18-alpine AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm install
COPY frontend/ .
RUN npm run build

# Build backend with regular glibc (not musl)
FROM --platform=linux/amd64 rustlang/rust:nightly AS backend-builder
WORKDIR /app

# Install OpenSSL development libraries
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/crates ./crates

# Build with regular target (not musl)
RUN cargo build --release

# Final runtime image - use debian (not alpine) to match glibc
FROM --platform=linux/amd64 debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=backend-builder /app/target/release/web_server .
COPY --from=frontend-builder /app/frontend/build ./frontend-build

EXPOSE 8080
CMD ["./web_server"]