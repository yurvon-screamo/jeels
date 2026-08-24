# syntax=docker/dockerfile:1.4

ARG RUST_VERSION=1.93

FROM ubuntu:26.04 AS builder

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

ARG RUST_VERSION

ARG ORIGA_LANDING_BASE_URL=https://origa.uwuwu.net
ENV ORIGA_LANDING_BASE_URL=${ORIGA_LANDING_BASE_URL}

ARG ORIGA_APP_BASE_URL
ENV ORIGA_APP_BASE_URL=${ORIGA_APP_BASE_URL}

ARG ORIGA_BUILD_DATE
ENV ORIGA_BUILD_DATE=${ORIGA_BUILD_DATE}

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    libssl-dev \
    musl-tools \
    pkg-config \
    && rm -rf /var/lib/apt/lists/* \
    && curl -fsSL https://github.com/mozilla/sccache/releases/download/v0.14.0/sccache-v0.14.0-x86_64-unknown-linux-musl.tar.gz | tar xz \
    && mv sccache-v0.14.0-x86_64-unknown-linux-musl/sccache /usr/local/bin/ \
    && chmod +x /usr/local/bin/sccache

ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/sccache

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain ${RUST_VERSION}
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

RUN rustup target add x86_64-unknown-linux-musl

# Copy ALL workspace Cargo.toml files (required for workspace resolution)
COPY Cargo.toml Cargo.lock ./
COPY utils/Cargo.toml ./utils/
COPY tauri/Cargo.toml ./tauri/
COPY tauri-plugin-aswebauth/Cargo.toml ./tauri-plugin-aswebauth/
COPY origa/Cargo.toml ./origa/
COPY origa_ui/Cargo.toml ./origa_ui/
COPY origa_landing/Cargo.toml ./origa_landing/
COPY .cargo .cargo

# Dummy source for ALL workspace members
RUN mkdir -p origa/src && echo "pub fn dummy() {}" > origa/src/lib.rs && \
    mkdir -p origa_ui/src && echo "pub fn dummy() {}" > origa_ui/src/lib.rs && \
    echo "fn main() {}" > origa_ui/src/main.rs && \
    mkdir -p utils/src && echo "pub fn dummy() {}" > utils/src/lib.rs && \
    mkdir -p tauri/src && echo "pub fn dummy() {}" > tauri/src/lib.rs && \
    mkdir -p tauri-plugin-aswebauth/src && echo "pub fn dummy() {}" > tauri-plugin-aswebauth/src/lib.rs && \
    echo "fn main() {}" > tauri-plugin-aswebauth/build.rs && \
    mkdir -p origa_landing/src && echo "pub fn dummy() {}" > origa_landing/src/lib.rs && \
    echo "fn main() {}" > origa_landing/src/main.rs && \
    echo "fn main() {}" > origa_landing/build.rs

# Dummy build - cache deps
RUN --mount=type=cache,target=/sccache,sharing=locked \
    --mount=type=cache,target=/root/.cargo/registry,sharing=locked \
    cargo build --release -p origa_landing --features ssr --target x86_64-unknown-linux-musl && \
    rm -rf origa_landing

# Real source
COPY origa_landing ./origa_landing

WORKDIR /app

# Real build - same pattern as origa_ui Dockerfile
RUN --mount=type=cache,target=/sccache,sharing=locked \
    --mount=type=cache,target=/root/.cargo/registry,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    rm -rf /app/target/x86_64-unknown-linux-musl/release/.fingerprint/origa_landing* && \
    rm -rf /app/target/x86_64-unknown-linux-musl/release/deps/origa_landing* && \
    rm -rf /app/target/x86_64-unknown-linux-musl/release/deps/liboriga_landing* && \
    rm -rf /app/target/x86_64-unknown-linux-musl/release/build/origa_landing* && \
    cargo build --release -p origa_landing --features ssr --target x86_64-unknown-linux-musl && \
    cp /app/target/x86_64-unknown-linux-musl/release/origa_landing /tmp/origa_landing

# Runtime
FROM alpine:3.24

WORKDIR /app

COPY --from=builder /tmp/origa_landing /usr/local/bin/origa_landing
COPY --from=builder /app/origa_landing/style/landing.processed.css /app/origa_landing/style/landing.processed.css
COPY --from=builder /app/origa_landing/public /app/origa_landing/public
COPY --from=builder /app/origa_landing/Cargo.toml /app/origa_landing/Cargo.toml

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["wget", "--spider", "-q", "http://localhost:3000/"]

CMD ["origa_landing"]
