# Build stage
FROM rust:1.86-alpine AS builder

RUN apk add --no-cache musl-dev sqlite-dev

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY src ./src
RUN cargo build --release

# Runtime stage
FROM alpine:3.21

RUN apk add --no-cache sqlite-libs

WORKDIR /app
RUN mkdir -p /data

COPY --from=builder /app/target/release/openshelf-backend .

EXPOSE 3001
CMD ["./openshelf-backend"]
