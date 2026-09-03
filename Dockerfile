FROM rust:1.89-bookworm

RUN apt-get update && \
    apt-get install -y ffmpeg python3 python3-pip pkg-config libssl-dev cmake && \
    pip3 install --break-system-packages yt-dlp && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY src ./src

RUN cargo build --release

CMD ["./target/release/icha-music-worker"]
