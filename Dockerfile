################################
ARG RUST_VERSION=1.91
ARG ALPINE_VERSION=3.22

################################
FROM rust:${RUST_VERSION}-alpine${ALPINE_VERSION} AS base
WORKDIR /app
RUN apk add --no-cache  musl-dev openssl-dev openssl-libs-static
RUN cargo install sccache
RUN cargo install cargo-chef
ENV CARGO_INCREMENTAL=0
ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/sccache
ENV SCCACHE_ERROR_LOG=/sccache.log

################################
FROM base AS planner
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo 'fn main() {}' > src/main.rs
RUN echo 'fn main() {}' > build.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef prepare --recipe-path recipe.json

################################
FROM base AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo build --release

################################
FROM alpine:${ALPINE_VERSION} AS runtime
WORKDIR /app

RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/echo-ip /echo-ip
ENTRYPOINT ["/app/echo-ip"]
