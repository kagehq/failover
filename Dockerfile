# --- Build (musl static) ---
FROM rust:1-alpine AS build
RUN apk add --no-cache musl-dev pkgconfig openssl-dev
WORKDIR /src
COPY . .
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

# --- Run ---
FROM alpine:3.20
RUN adduser -D -H app
USER app
WORKDIR /app
COPY --from=build /src/target/x86_64-unknown-linux-musl/release/failover-proxy /usr/local/bin/failover-proxy
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/failover-proxy"]
