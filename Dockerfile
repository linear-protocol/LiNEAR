FROM rust:1.69.0
LABEL description="Linear contract builder"

RUN apt-get update && apt-get install -y git less vim clang

RUN rustup default 1.69.0
RUN rustup target add wasm32-unknown-unknown

# Cargo files are needed to install and cache dependencies
ADD contracts/linear/Cargo.toml .
ADD Cargo.lock .

# a trick to run cargo build
RUN mkdir -p src && echo "fn main() {}" > src/lib.rs && \ 
        RUSTFLAGS="-C link-arg=-s" cargo build -p linear --target wasm32-unknown-unknown --release && \
        rm Cargo.toml && \
        rm -rf ./src/ target/release/deps/my-project* target/release/my-project*
