# LiNEAR
NEAR Liquid Staking

## Build
- Build all artifacts: `make`
- Build linear only: `make linear`
- Build mockup staking-pool: `make mock-staking-pool`

## Test
- Run all tests: `make test`
- Run mock staking-pool tests: `make test-mock-staking-pool`
- Run LiNEAR simulation tests:
  - Run all: `make test-linear`
  - Run specific test file: `TEST_FILE={filename} make test-linear`
  - Print contract logs: `LOGS=1 make test-linear`

## Design

### Terminologies
- `total_share_amount`: Total amount of LiNEAR that was minted (minus burned).
- `total_staked_near_amount`: Total amount of NEAR that was staked by users to this contract.     
  This is effectively 1) amount of NEAR that was deposited to this contract but hasn't yet been staked on any validators + 2) amount of NEAR that has already been staked on validators.    
  Note that the amount of NEAR that is pending release or is already released by hasn't been withdrawn is not considered.
- `share_price`: how much NEAR does one share (LiNEAR) worth. equals to : `total_near_amount` / `total_share_amount`.
