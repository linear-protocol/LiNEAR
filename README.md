# LiNEAR - Liquid Staking on NEAR Protocol

LiNEAR Protocol is a liquid staking solution built on the NEAR Protocol. LiNEAR unlocks liquidity of the staked NEAR by creating a staking derivative to be engaged with various DeFi protocols on NEAR and Aurora, while also enjoying over 10% APY staking rewards of the underlying base tokens. LiNEAR is the cornerstone piece of the NEAR-Aurora DeFi ecosystem.

- [Documentation](https://docs.linearprotocol.org/)

## Contracts

The LiNEAR smart contracts are implemented with [NEAR Rust SDK](https://near-sdk.io/). The core contract is located in `contracts/linear`, and several mock contracts were made for testing various scenarios via simulation test.

The [v1.0.0 contract release](https://github.com/linear-protocol/LiNEAR/releases/tag/v1.0.0) has been audited by [BlockSec](https://www.blocksecteam.com/). According to [BlockSec's auditing report](https://github.com/linear-protocol/audits/blob/main/BlockSec%20-%20Security%20Audit%20Report%20for%20LiNEAR%20-%20202204.pdf), no critical issues were found, and few low-risk minor issues were reported and have been fixed.

## Deployment

### Mainnet

- LiNEAR contract and $LiNEAR token: [`linear-protocol.near`](https://explorer.near.org/accounts/linear-protocol.near)

### Testnet

- LiNEAR contract and $LiNEAR token: [`linear-protocol.testnet`](https://explorer.testnet.near.org/accounts/linear-protocol.testnet)


## Development


### Build
- Build all artifacts: `make`
- Build linear only: `make linear`

### Test

We adopt unit tests and heavily used the [`workspace-js`](https://github.com/near/workspaces-js) test framework to test the major scenarios and workflow of the LiNEAR smart contract in the [Sandbox](https://docs.near.org/docs/develop/contracts/sandbox) environment. Lint with `rustfmt` and `clippy` is also required when making changes to contract.

- Run `npm i` under `./tests` folder first to set up the environment
- Run lint with `rustfmt` and `clippy`: `make lint`
- Run all tests: `make test`
- Run LiNEAR simulation tests:
  - Run all: `make test-linear`
  - Run specific test file: `TEST_FILE={filename} make test-linear`
  - Print contract logs: `LOGS=1 make test-linear`

### Deploy

A CLI tool is made to help deploy and manage the contract.
In order to use it, run `npm i` first.        

*All CLI arguments could be passed as environment variables as well, with `LI` prefix.*

- deploy contract:
  - `./bin/cli.js deploy linear.testnet`

- init contract:
  - `./bin/cli.js init linear.testnet --signer owner.testnet --owner_id owner.testnet`

### Release

- Checkout a release branch from main
  - `git checkout -b release/v1.0.x`
- Make sure all tests pass:
  - `make test`
- Update version in `contracts/linear/Cargo.toml`
- Git commit version update and push the release branch:
  - `git commit -m 'v1.0.x'`
  - `git push origin release/v1.0.x`
- Create a PR from the release branch
- Once the PR is merged, publish a new release on GitHub


## Manage
### Drain
Drain is to totally remove a validator from candidate list, all funds on it will be re-distributed
among others.

1. Make sure there is currently no unstaked balance on it. If there is, call `epoch_withdraw` to withdraw.
2. Set validator weight to 0, which can be done by either removing this validator from nodes list or set its weight to 0 directly. Run `set-node` command to update the weight.
3. Run `drain-unstake` to unstake all funds from the validator.
4. After 4 epoches, run `drain-withdraw` to withdraw and restake those funds.

## Design

### Terminologies
- `total_share_amount`: Total amount of LiNEAR that was minted (minus burned).
- `total_staked_near_amount`: Total amount of NEAR that was staked by users to this contract.     
  This is effectively 1) amount of NEAR that was deposited to this contract but hasn't yet been staked on any validators + 2) amount of NEAR that has already been staked on validators.    
  Note that the amount of NEAR that is pending release or is already released by hasn't been withdrawn is not considered.
- `stake_share_price`: how much NEAR does one stake share (LiNEAR) worth. equals to : `total_near_amount` / `total_share_amount`.


## Information for Developers

More useful information for developers who're building with LiNEAR

### LiNEAR App URLs

- mainnet: `app.linearprotocol.org`
- testnet: `testnet.linearprotocol.org`

### LiNEAR contract addresses on NEAR: 

- mainnet: `linear-protocol.near`
- testnet: `linear-protocol.testnet`

### LiNEAR ERC20 contracts on Aurora (bridged via Rainbow Bridge)

- mainnet: `0x918dbe087040a41b786f0da83190c293dae24749`
- testnet: `0xe4979cac5d70f01697f795f0ad56bbca05912c44`

Data source: https://github.com/aurora-is-near/bridge-assets/tree/master/tokens

### LiNEAR Oracles on Aurora mainnet:

- LINEAR / NEAR: `0x8f975aC6deFD2c9d50c58BABF4B1f880E6dE7996`
- LINEAR / USD: `0x2eBf49106814Fcd8685ed6c8a7315Ca528CdA232`

More details can be found from Flux docs: https://docs.fluxprotocol.org/docs/live-data-feeds/fpo-live-networks-and-pairs
