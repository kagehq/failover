# --- Build (musl static) ---
FROM rust:1-alpine AS build
RUN apk add --no-cache musl-dev pkgconfig openssl-dev
WORKDIR /src
COPY . .
# Build for the native target (Alpine is already musl-based)
RUN cargo build --release

# --- Run ---
FROM alpine:3.20
RUN adduser -D -H app
USER app
WORKDIR /app
COPY --from=build /src/target/release/failover /usr/local/bin/failover
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/failover"]
