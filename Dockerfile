# syntax=docker/dockerfile:1

# Build the front end into static files.
FROM node:22-bookworm-slim AS web
WORKDIR /web
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ ./
RUN npm run build

# Build the Rust binary. SQLX_OFFLINE uses the committed .sqlx/ query data, so no database is needed.
FROM rust:1.86-bookworm AS build
WORKDIR /src
ENV SQLX_OFFLINE=true
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked --bin team \
    && install -D target/release/team /out/team

# The app. Runs as user 1000 with a read-only root file system (cluster contract, see README.md).
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /out/team /app/team
COPY --from=web /web/build /app/web
USER 1000:1000
ENV HOST=0.0.0.0 PORT=3000
EXPOSE 3000
CMD ["/app/team", "serve"]

# Runs as an init container before the app on every pod start.
FROM runtime AS migration
CMD ["/app/team", "migrate"]
