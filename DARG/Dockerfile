# --- Etapa 1: Compilador del Sistema (Builder) ---
FROM rust:slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    build-essential \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src

COPY . .

# Compilamos únicamente coordinator y worker
RUN cargo build --release -p coordinator -p worker

# --- Etapa 2: Imagen Final del Worker ---
FROM ubuntu:24.04

# 1. Herramientas del sistema y dependencias básicas (sin cargo/rustc de apt)
RUN apt-get update && apt-get install -y \
    curl \
    pkg-config \
    ca-certificates \
    libsqlite3-0 \
    libssl-dev \
    libsqlite3-dev \
    git \
    build-essential \
    default-jdk \
    maven \
    unzip \
    zip \
    && rm -rf /var/lib/apt/lists/*

# 2. Instalamos Rust y Cargo actualizados mediante Rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# 3. Configuramos el PATH para usar Rust/Cargo de Rustup
ENV PATH="/root/.cargo/bin:${PATH}"

# 4. Instalamos el componente cargo fmt (rustfmt)
RUN rustup component add rustfmt

WORKDIR /usr/src

# Copiamos los binarios
COPY --from=builder /usr/src/target/release/coordinator ./target/release/coordinator
COPY --from=builder /usr/src/target/release/worker ./target/release/worker

CMD ["./target/release/worker"]