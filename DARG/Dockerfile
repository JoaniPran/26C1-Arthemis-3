FROM rust:slim

RUN apt-get update && apt-get install -y \
    git \
    build-essential \
    zip \
    unzip \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src

COPY ./common ./common
COPY ./worker ./worker

WORKDIR /usr/src/worker
RUN cargo build --release

CMD ["./target/release/worker"]