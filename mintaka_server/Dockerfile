FROM rust:alpine AS builder

RUN apk add --no-cache build-base

WORKDIR /app

COPY . .

RUN rustup toolchain install --profile minimal

# cached build
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --locked --release -p mintaka_server \
    && cp /app/target/release/mintaka_server /tmp/mintaka_server

FROM alpine AS runtime

RUN apk --no-cache add curl

WORKDIR /var/lib/mintaka

COPY --from=builder /tmp/mintaka_server /usr/local/bin/mintaka_server
COPY mintaka_webui mintaka_webui

RUN mkdir -p /var/lib/mintaka/sessions \
    && chown -R 10001:10001 /var/lib/mintaka

USER 10001:10001

ENTRYPOINT ["/usr/local/bin/mintaka_server"]
