RFLAGS="-C link-arg=-s"

all: linear

release:
	$(call docker_build)
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/linear.wasm ./res/linear.wasm

linear: contracts/linear check-rustc-version
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p linear --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/linear.wasm ./res/linear.wasm

linear_test: contracts/linear check-rustc-version
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

mock-dex: contracts/mock-dex
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p mock-dex --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/mock_dex.wasm ./res/mock_dex.wasm

mock-lockup: contracts/mock-lockup
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p mock-lockup --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/mock_lockup.wasm ./res/mock_lockup.wasm

mock-whitelist: contracts/mock-whitelist
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p mock-whitelist --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/mock_whitelist.wasm ./res/mock_whitelist.wasm

clean:
	rm res/*.wasm

lint:
	cargo fmt -- --check
	cargo clippy --tests -- -D clippy::all

test: test-unit test-linear test-mock-staking-pool test-mock-fungible-token

test-unit:
	cargo test --features "test"

TEST_FILE ?= **
LOGS ?=
test-linear: linear_test mock-staking-pool mock-fungible-token mock-dex mock-lockup mock-whitelist
	@mkdir -p ./tests/compiled-contracts/
	@cp ./res/linear_test.wasm ./tests/compiled-contracts/linear.wasm
	@cp ./res/mock_staking_pool.wasm ./tests/compiled-contracts/mock_staking_pool.wasm
	@cp ./res/mock_fungible_token.wasm ./tests/compiled-contracts/mock_fungible_token.wasm
	@cp ./res/mock_dex.wasm ./tests/compiled-contracts/mock_dex.wasm
	@cp ./res/mock_lockup.wasm ./tests/compiled-contracts/mock_lockup.wasm
	@cp ./res/mock_whitelist.wasm ./tests/compiled-contracts/mock_whitelist.wasm
	cd tests && NEAR_PRINT_LOGS=$(LOGS) npx near-workspaces-ava --timeout=2m __tests__/linear/$(TEST_FILE).ava.ts --verbose

test-mock-staking-pool: mock-staking-pool
	@mkdir -p ./tests/compiled-contracts/
	cp ./res/mock_staking_pool.wasm ./tests/compiled-contracts/mock_staking_pool.wasm
	cd tests && npx near-workspaces-ava __tests__/mock-staking-pool/**.ts --verbose

test-mock-fungible-token: mock-fungible-token
	@mkdir -p ./tests/compiled-contracts/
	cp ./res/mock_fungible_token.wasm ./tests/compiled-contracts/mock_fungible_token.wasm
	cd tests && npx near-workspaces-ava __tests__/mock-fungible-token/**.ts --verbose

check-rustc-version:
	@RUSTC_VERSION=$$(rustc --version | awk '{print $$2}'); \
	if [ "$$RUSTC_VERSION" != "1.68.0" ]; then \
		echo "Error: Rustc version is $$RUSTC_VERSION but 1.68.0 is required." && exit 1; \
	else \
		echo "Rustc version $$RUSTC_VERSION is installed."; \
	fi

define docker_build
	docker build -t near-builder:1.68.0 .
	docker run \
		--mount type=bind,source=${PWD},target=/host \
		--cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
		-w /host \
		-e RUSTFLAGS=$(RFLAGS) \
		-i -t near-builder:1.68.0 \
		make
endef
