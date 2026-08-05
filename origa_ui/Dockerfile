# syntax=docker/dockerfile:1.4

ARG RUST_VERSION=1.93
ARG TRUNK_VERSION=0.21.14

FROM ubuntu:26.04 AS builder

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

ARG RUST_VERSION
ARG TRUNK_VERSION
ARG TAILWINDCSS_VERSION=3.4.17

ARG ORIGA_VERSION=dev
ARG ORIGA_COMMIT=unknown
ARG ORIGA_BUILD_DATE=unknown
ARG ORIGA_CDN_BASE_URL
ARG ORIGA_CDN_REGION=auto
ARG TRAILBASE_URL=http://localhost:8080

ENV ORIGA_VERSION=${ORIGA_VERSION}
ENV ORIGA_COMMIT=${ORIGA_COMMIT}
ENV ORIGA_BUILD_DATE=${ORIGA_BUILD_DATE}
ENV ORIGA_CDN_BASE_URL=${ORIGA_CDN_BASE_URL}
ENV ORIGA_CDN_REGION=${ORIGA_CDN_REGION}
ENV TRAILBASE_URL=${TRAILBASE_URL}

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    binaryen \
    build-essential \
    ca-certificates \
    clang \
    curl \
    libssl-dev \
    lld \
    nodejs \
    npm \
    pkg-config \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/* \
    && curl -fsSL https://github.com/mozilla/sccache/releases/download/v0.14.0/sccache-v0.14.0-x86_64-unknown-linux-musl.tar.gz | tar xz \
    && mv sccache-v0.14.0-x86_64-unknown-linux-musl/sccache /usr/local/bin/ \
    && chmod +x /usr/local/bin/sccache

ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/sccache

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain ${RUST_VERSION}
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

RUN rustup target add wasm32-unknown-unknown && \
    curl -fsSL https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz \
    | tar -xzf - -C /usr/local/bin

RUN npm install -g tailwindcss@${TAILWINDCSS_VERSION} \
    && tailwindcss --help > /dev/null

# Copy ALL workspace Cargo.toml files (required for workspace resolution)
COPY Cargo.toml Cargo.lock ./
COPY utils/Cargo.toml ./utils/
COPY tauri/Cargo.toml ./tauri/
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
    mkdir -p origa_landing/src && echo "pub fn dummy() {}" > origa_landing/src/lib.rs && \
    echo "fn main() {}" > origa_landing/src/main.rs && \
    echo "fn main() {}" > origa_landing/build.rs

# Dummy build - only origa_ui (wasm)
RUN --mount=type=cache,target=/sccache,sharing=locked \
    --mount=type=cache,target=/root/.cargo/registry,sharing=locked \
    cargo build --release -p origa_ui --target wasm32-unknown-unknown && \
    rm -rf origa origa_ui

# Real source
COPY origa ./origa
COPY origa_ui ./origa_ui
COPY build_defaults.rs ./

# Real build - origa + origa_ui (wasm)
RUN --mount=type=cache,target=/sccache,sharing=locked \
    --mount=type=cache,target=/root/.cargo/registry,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    rm -rf origa_ui/public/dictionaries/unidic && \
    rm -rf /app/target/wasm32-unknown-unknown/release/.fingerprint/origa* && \
    rm -rf /app/target/wasm32-unknown-unknown/release/deps/origa* && \
    rm -rf /app/target/wasm32-unknown-unknown/release/deps/liboriga* && \
    rm -rf /app/target/wasm32-unknown-unknown/release/build/origa* && \
    cargo build --release -p origa --target wasm32-unknown-unknown && \
    cargo build --release -p origa_ui --target wasm32-unknown-unknown

# Trunk build
WORKDIR /app/origa_ui

RUN --mount=type=cache,target=/root/.cargo/registry,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    --mount=type=cache,target=/root/.cache/trunk,sharing=locked \
    trunk build --release --dist dist

# Runtime: TrailBase + SPA static files
FROM trailbase/trailbase:0.31.1

USER root
WORKDIR /app

ARG ORIGA_CDN_BASE_URL
ENV ORIGA_CDN_BASE_URL=${ORIGA_CDN_BASE_URL}

# hadolint ignore=DL3018
RUN apk add --no-cache curl sqlite aws-cli

COPY --from=builder /app/origa_ui/dist /app/public

# Backup machinery: GFS-rotation script, custom entrypoint (crond + trail), crontab
COPY backup.sh     /app/backup.sh
COPY entrypoint.sh /app/entrypoint.sh
COPY crontab       /app/crontab
RUN chmod +x /app/backup.sh /app/entrypoint.sh \
 && chmod 600 /app/crontab

RUN chown -R trailbase:trailbase /app

USER trailbase

ENV BACKUP_BUCKET_PATH=trailbase

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["curl", "--fail", "http://localhost:8080/api/healthcheck"]

ENTRYPOINT ["/app/entrypoint.sh"]
CMD ["/app/trail", "run", "--address", "0.0.0.0:8080", "--public-dir", "/app/public", "--spa"]
