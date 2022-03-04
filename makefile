RFLAGS="-C link-arg=-s"

all: linear mock-staking-pool

linear: contracts/linear
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p linear --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/linear.wasm ./res/linear.wasm

linear_test: contracts/linear
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p linear --target wasm32-unknown-unknown --features "test"
	mkdir -p res
	cp target/wasm32-unknown-unknown/debug/linear.wasm ./res/linear_test.wasm

mock-staking-pool: contracts/mock-staking-pool
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p mock-staking-pool --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/mock_staking_pool.wasm ./res/mock_staking_pool.wasm

mock-fungible-token: contracts/mock-fungible-token
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p mock-fungible-token --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/mock_fungible_token.wasm ./res/mock_fungible_token.wasm

clean:
	rm res/*.wasm

test: test-unit test-linear test-mock-staking-pool

test-unit:
	cargo test --features "test"

TEST_FILE ?= **
LOGS ?=
test-linear: linear_test mock-staking-pool mock-fungible-token
	@mkdir -p ./tests/compiled-contracts/
	cp ./res/linear_test.wasm ./tests/compiled-contracts/linear.wasm
	cp ./res/mock_staking_pool.wasm ./tests/compiled-contracts/mock_staking_pool.wasm
	cp ./res/mock_fungible_token.wasm ./tests/compiled-contracts/mock_fungible_token.wasm
	cd tests && NEAR_PRINT_LOGS=$(LOGS) npx near-workspaces-ava --timeout=2m __tests__/linear/$(TEST_FILE).ava.ts --verbose

test-mock-staking-pool: mock-staking-pool
	@mkdir -p ./tests/compiled-contracts/
	cp ./res/mock_staking_pool.wasm ./tests/compiled-contracts/mock_staking_pool.wasm
	cd tests && npx near-workspaces-ava __tests__/mock-staking-pool/**.ts --verbose

test-mock-fungible-token: mock-fungible-token
	@mkdir -p ./tests/compiled-contracts/
	cp ./res/mock_fungible_token.wasm ./tests/compiled-contracts/mock_fungible_token.wasm
	cd tests && npx near-workspaces-ava __tests__/mock-fungible-token/**.ts --verbose
