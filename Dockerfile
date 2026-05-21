# --- Étape 1 : Compilation ---
FROM rust:1.95-slim AS builder

WORKDIR /usr/src/app

# Dépendances de build
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copie du projet complet (workspace)
COPY . .

# Compilation release du backend uniquement
RUN cargo build --release -p backend

# --- Étape 2 : Image d'exécution légère ---
FROM debian:bookworm-slim

# OpenSSL + certificats — requis pour NeonDB (TLS) et JWT
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/app/target/release/backend ./backend

# Railway injecte $PORT dynamiquement (généralement 8080)
EXPOSE 8080

CMD ["./backend"]