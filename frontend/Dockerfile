# Build stage
FROM oven/bun:1-alpine AS builder

WORKDIR /app
COPY package.json .
RUN bun install

COPY . .
RUN bun run build

# Runtime stage — serve with nginx
FROM nginx:1.27-alpine

COPY --from=builder /app/build /usr/share/nginx/html

EXPOSE 80
