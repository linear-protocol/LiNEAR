RFLAGS="-C link-arg=-s"

all: linear staking-pool

linear: contracts/linear
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p linear --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/linear.wasm ./res/linear.wasm

staking-pool: contracts/staking-pool
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p staking-pool --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/staking_pool.wasm ./res/staking_pool.wasm

clean:
	rm res/*.wasm
