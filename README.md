# LiNEAR - Liquid Staking on NEAR Protocol

LiNEAR Protocol is a liquid staking solution built on the NEAR Protocol. LiNEAR unlocks liquidity of the staked NEAR by creating a staking derivative to be engaged with various DeFi protocols on NEAR and Aurora, while also enjoying over 10% APY staking rewards of the underlying base tokens. LiNEAR is the cornerstone piece of the NEAR-Aurora DeFi ecosystem.

- [Documentation](https://docs.linearprotocol.org/)

## Contracts

The LiNEAR smart contracts are implemented with [NEAR Rust SDK](https://near-sdk.io/). The core contract is located in `contracts/linear`, and several mock contracts were made for testing various scenarios via simulation test.

The [v1.0.0 release](https://github.com/linear-protocol/LiNEAR/releases/tag/v1.0.0) has been audited by [BlockSec](https://www.blocksecteam.com/). According to [its report](https://files.gitbook.com/v0/b/gitbook-x-prod.appspot.com/o/spaces%2FVUsjtIZTjrEX8y9oBG0Z%2Fuploads%2FANndiBCKoL0hX9HNEUd7%2FLiNEAR%20security%20auditing%20by%20BlockSec.pdf?alt=media&token=655abd1f-50d3-40b4-8d0b-a38897d98392), no critical issues were found, and several low-risk minor issues were reported and have been fixed.

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

We adopt unit tests and heavily used the [`workspace-js`](https://github.com/near/workspaces-js) test framework to test the major scenarios and workflow of the LiNEAR smart contract in the [Sandbox](https://docs.near.org/docs/develop/contracts/sandbox) environment

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
### - Drain
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
