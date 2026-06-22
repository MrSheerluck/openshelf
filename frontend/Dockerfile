# Build stage
FROM oven/bun:1-alpine AS builder

WORKDIR /app
COPY package.json .
RUN bun install

COPY . .
ARG VITE_API_URL
ENV VITE_API_URL=${VITE_API_URL}
RUN bun run build

# Runtime stage — serve with nginx
FROM nginx:1.27-alpine

COPY --from=builder /app/build /usr/share/nginx/html

EXPOSE 80
