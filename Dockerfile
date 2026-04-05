FROM rust:1.92-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Create a dummy project to cache dependencies
# This trick avoids recompiling all dependencies when only source code changes
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy the actual source code
COPY src ./src

# Touch main.rs to invalidate the binary cache (but not dependencies)
RUN touch src/main.rs 

# Build the actual binary
RUN cargo build --release

FROM debian:trixie-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/tetronix_backend /usr/local/bin/tetronix_backend

EXPOSE 8080

CMD ["tetronix_backend"]
