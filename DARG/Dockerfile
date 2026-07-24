# --- Etapa 1: Compilador (Builder) ---
FROM rust:slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src

# Copiamos TODO el proyecto (incluye ui, common, coordinator, worker y manifiestos)
COPY . .

# Compilamos ÚNICAMENTE coordinator y worker (ignora la ui en la compilación)
RUN cargo build --release -p coordinator -p worker

# --- Etapa 2: Imagen Final (Liviana) ---
FROM ubuntu:24.04

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src

# Copiamos los binarios resultantes
COPY --from=builder /usr/src/target/release/coordinator ./target/release/coordinator
COPY --from=builder /usr/src/target/release/worker ./target/release/worker

CMD ["./target/release/worker"]