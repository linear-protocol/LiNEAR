FROM rust:1.68.0
LABEL description="Container for builds"

ENV RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
ENV RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup

RUN apt-get update && apt-get install -y git less vim clang

RUN rustup default 1.68.0
RUN rustup target add wasm32-unknown-unknown

ADD contracts/linear/Cargo.toml .
ADD Cargo.lock .

RUN mkdir -p src && echo "fn main() {}" > src/lib.rs && \ 
        RUSTFLAGS="-C link-arg=-s" cargo build -p linear --target wasm32-unknown-unknown --release && \
        rm Cargo.toml && \
        rm -rf ./src/ target/release/deps/my-project* target/release/my-project*
