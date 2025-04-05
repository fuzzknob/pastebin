FROM rust:1.85-alpine3.20 AS builder

RUN apk add --no-cache musl-dev openssl-dev

WORKDIR /usr/src/app
RUN USER=root cargo new --bin pastebin
WORKDIR /usr/src/app/pastebin

COPY Cargo.lock ./
COPY Cargo.toml ./

RUN cargo build --release
RUN rm src/*.rs

COPY src ./src
COPY templates ./templates

RUN rm ./target/release/deps/pastebin*
RUN cargo build --release

# Final stage
FROM alpine:3.20

# # Install necessary runtime dependencies
# RUN apt-get update && apt-get install -y --no-install-recommends \
#     ca-certificates \
#     && rm -rf /var/lib/apt/lists/*
RUN apk add --no-cache ca-certificates

COPY --from=builder /usr/src/app/pastebin/target/release/pastebin /usr/local/bin/pastebin

# Set the startup command
ENV RUST_LOG=info
EXPOSE 8000
CMD ["pastebin"]
