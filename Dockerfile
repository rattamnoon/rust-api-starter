FROM rust:1.95.0-bookworm AS dev

RUN cargo install cargo-watch

WORKDIR /app

FROM rust:1.95.0-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src

RUN cargo build --release --bin rust-api-starter

FROM debian:bookworm-slim AS runtime

ENV APP_HOME=/app \
    LOG_DIR=/app/logs \
    UPLOAD_DIR=/app/uploads \
    SERVER_HOST=0.0.0.0 \
    SERVER_PORT=8080

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system app \
    && useradd --system --gid app --create-home --home-dir "${APP_HOME}" app

WORKDIR ${APP_HOME}

COPY --from=builder /app/target/release/rust-api-starter ./rust-api-starter

RUN mkdir -p "${LOG_DIR}" "${UPLOAD_DIR}" \
    && chown -R app:app "${APP_HOME}"

USER app

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD sh -c 'curl --fail "http://127.0.0.1:${SERVER_PORT}/api/v1/health" >/dev/null || exit 1'

CMD ["./rust-api-starter"]
