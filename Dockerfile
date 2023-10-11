FROM rust:1.73.0-slim-bookworm as builder

WORKDIR /usr/src/linkers

COPY ./ ./

RUN apt update  \
    && apt upgrade -y \
    && apt install libssl-dev pkg-config -y --no-install-recommends  \
    && rm -rf /var/lib/apt/lists/*

RUN cargo build --release

CMD ["./target/release/linkers"]