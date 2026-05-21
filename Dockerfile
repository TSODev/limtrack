# --- Étape 1 : Compilation ---
FROM rust:1.95-slim AS builder

WORKDIR /usr/src/app

# Install build dependencies (pkg-config and OpenSSL development headers)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# On copie l'ensemble du projet (y compris le dossier backend)
COPY . .

# Compilation en mode Release pour des performances maximales
RUN cargo build --release -p backend

# --- Étape 2 : Image d'exécution (ultra-légère) ---
FROM debian:bookworm-slim

# Installation des certificats SSL (indispensable pour communiquer avec la DB Neon en HTTPS)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# On récupère le binaire compilé depuis l'étape builder
COPY --from=builder /usr/src/app/target/release/backend ./backend

# Railway injecte automatiquement une variable d'environnement PORT.
EXPOSE 8080

CMD ["./backend"]