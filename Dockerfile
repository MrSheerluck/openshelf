FROM oven/bun:1 AS frontend
WORKDIR /app
COPY frontend/package.json frontend/bun.lock ./
RUN bun install --frozen-lockfile
COPY frontend/ ./
ENV VITE_API_URL=
RUN bun run build

FROM rust:1-slim AS backend
RUN apt-get update && apt-get install -y build-essential && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src
COPY backend/src/ ./src/
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=caddy:2-alpine /usr/bin/caddy /usr/local/bin/caddy
COPY --from=backend /app/target/release/openshelf-backend /usr/local/bin/openshelf-backend
COPY --from=frontend /app/build /srv/frontend
COPY Caddyfile /etc/caddy/Caddyfile
COPY docker-entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENV DATABASE_PATH=/data/app.db
ENV HOST=0.0.0.0
ENV PORT=3001
ENV XDG_DATA_HOME=/data
VOLUME /data
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 CMD curl -f http://localhost:3001/api/health || exit 1
ENTRYPOINT ["/entrypoint.sh"]
