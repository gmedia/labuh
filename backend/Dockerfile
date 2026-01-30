FROM rust:1.83-alpine AS builder

RUN apk add --no-cache musl-dev pkgconfig openssl-dev

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Create dummy main.rs for caching dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (cached layer)
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM alpine:3.20

RUN apk add --no-cache ca-certificates libgcc

WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/labuh /app/labuh

# Copy migrations
COPY migrations ./migrations

# Create data directory
RUN mkdir -p /data

EXPOSE 3000

ENV RUST_LOG=labuh=info,tower_http=info

CMD ["/app/labuh"]
