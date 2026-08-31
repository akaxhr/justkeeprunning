FROM rust:1.89-bookworm

RUN apt-get update && \
    apt-get install -y ffmpeg pkg-config libssl-dev cmake && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY test.mp3 /app/test.mp3

RUN cargo build --release

CMD ["./target/release/icha-music-worker"]
