FROM rust:1.88-bookworm

RUN apt-get update && \
    apt-get install -y ffmpeg pkg-config libssl-dev git cmake && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN git clone https://github.com/ankit-chaubey/tgcalls.git

WORKDIR /app/tgcalls

RUN cargo build --release --example group_audio_call

WORKDIR /app

COPY src ./src

CMD ["./tgcalls/target/release/examples/group_audio_call"]
