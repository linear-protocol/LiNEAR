use crate::events::Event;
use crate::types::*;
use crate::*;
use near_contract_standards::fungible_token::events::{FtBurn, FtMint};
use near_sdk::{log, Promise};
use std::collections::HashMap;

impl LiquidStakingContract {
    /********************/
    /* Internal methods */
    /********************/
    pub(crate) fn assert_running(&self) {
        require!(!self.paused, ERR_PAUSED);
    }

    pub(crate) fn internal_deposit(&mut self, amount: Balance) {
        self.assert_running();

        let account_id = env::predecessor_account_id();
        let mut account = self.internal_get_account(&account_id);
        account.unstaked += amount;
        self.internal_save_account(&account_id, &account);

        Event::Deposit {
            account_id: &account_id,
            amount: &U128(amount),
            new_unstaked_balance: &U128(account.unstaked),
        }
        .emit();
    }

    pub(crate) fn assert_can_withdraw(&self, account_id: &AccountId, amount: Balance) {
        require!(amount > 0, ERR_NON_POSITIVE_WITHDRAWAL_AMOUNT);

        let account = self.internal_get_account(account_id);
        require!(
            account.unstaked >= amount,
            ERR_NO_ENOUGH_UNSTAKED_BALANCE_TO_WITHDRAW
        );
        require!(
            account.unstaked_available_epoch_height <= get_epoch_height(),
            ERR_UNSTAKED_BALANCE_NOT_AVAILABLE
        );
        // Make sure the contract has enough NEAR for user to withdraw,
        // Note that account locked balance should not be included.
        let available_balance = env::account_balance();
        // at least 1 NEAR should be left to cover storage/gas.
        require!(
            available_balance.saturating_sub(CONTRACT_MIN_RESERVE_BALANCE) >= amount,
            ERR_NO_ENOUGH_CONTRACT_BALANCE
        );
    }

    pub(crate) fn internal_withdraw(&mut self, amount: Balance) {
        self.assert_running();

        let account_id = env::predecessor_account_id();
        self.assert_can_withdraw(&account_id, amount);

        let mut account = self.internal_get_account(&account_id);
        account.unstaked -= amount;
        self.internal_save_account(&account_id, &account);

        Event::Withdraw {
            account_id: &account_id,
            amount: &U128(amount),
            new_unstaked_balance: &U128(account.unstaked),
        }
        .emit();
        Promise::new(account_id).transfer(amount);
    }

    pub(crate) fn internal_stake(&mut self, amount: Balance) -> ShareBalance {
        self.assert_running();

        require!(amount > 0, ERR_NON_POSITIVE_STAKING_AMOUNT);

        let account_id = env::predecessor_account_id();
        let mut account = self.internal_get_account(&account_id);

        // Calculate the number of "stake" shares that the account will receive for staking the
        // given amount.
        let num_shares = self.num_shares_from_staked_amount_rounded_down(amount);
        require!(num_shares > 0, ERR_NON_POSITIVE_CALCULATED_STAKING_SHARE);
        // The amount of tokens the account will be charged from the unstaked balance.
        // Rounded down to avoid overcharging the account to guarantee that the account can always
        // unstake at least the same amount as staked.
        let charge_amount = self.staked_amount_from_num_shares_rounded_down(num_shares);
        require!(charge_amount > 0, ERR_NON_POSITIVE_CALCULATED_STAKED_AMOUNT);

        require!(
            account.unstaked >= charge_amount,
            ERR_NO_ENOUGH_UNSTAKED_BALANCE
        );
        account.unstaked -= charge_amount;
        account.stake_shares += num_shares;
        self.internal_save_account(&account_id, &account);

        // The staked amount that will be added to the total to guarantee the "stake" share price
        // never decreases. The difference between `stake_amount` and `charge_amount` is paid
        // from the allocated STAKE_SHARE_PRICE_GUARANTEE_FUND.
        let stake_amount = self.staked_amount_from_num_shares_rounded_up(num_shares);

        self.total_staked_near_amount += stake_amount;
        self.total_share_amount += num_shares;

        // Increase requested stake amount within the current epoch
        self.epoch_requested_stake_amount += stake_amount;

        Event::Stake {
            account_id: &account_id,
            staked_amount: &U128(charge_amount),
            minted_stake_shares: &U128(num_shares),
            new_unstaked_balance: &U128(account.unstaked),
            new_stake_shares: &U128(account.stake_shares),
        }
        .emit();
        FtMint {
            owner_id: &account_id,
            amount: &U128(num_shares),
            memo: Some("stake"),
        }
        .emit();
        log!(
            "Contract total staked balance is {}. Total number of shares {}",
            self.total_staked_near_amount,
            self.total_share_amount
        );

        num_shares
    }

    pub(crate) fn internal_unstake(&mut self, amount: u128) {
        self.assert_running();

        require!(amount > 0, ERR_NON_POSITIVE_UNSTAKING_AMOUNT);

        let account_id = env::predecessor_account_id();
        let mut account = self.internal_get_account(&account_id);

        require!(
            self.total_staked_near_amount > 0,
            ERR_CONTRACT_NO_STAKED_BALANCE
        );
        // Calculate the number of shares required to unstake the given amount.
        // NOTE: The number of shares the account will pay is rounded up.
        let num_shares = self.num_shares_from_staked_amount_rounded_up(amount);
        require!(num_shares > 0, ERR_NON_POSITIVE_CALCULATED_UNSTAKING_SHARE);
        require!(
            account.stake_shares >= num_shares,
            ERR_NO_ENOUGH_STAKED_BALANCE
        );

        // Calculating the amount of tokens the account will receive by unstaking the corresponding
        // number of "stake" shares, rounding up.
        let receive_amount = self.staked_amount_from_num_shares_rounded_up(num_shares);
        require!(
            receive_amount > 0,
            ERR_NON_POSITIVE_CALCULATED_STAKED_AMOUNT
        );

        account.stake_shares -= num_shares;
        account.unstaked += receive_amount;
        account.unstaked_available_epoch_height =
            get_epoch_height() + self.validator_pool.get_num_epoch_to_unstake(amount);
        if self.last_settlement_epoch == get_epoch_height() {
            // The unstake request is received after epoch_cleanup
            // so actual unstake will happen in the next epoch,
            // which will put withdraw off for one more epoch.
            account.unstaked_available_epoch_height += 1;
        }

        self.internal_save_account(&account_id, &account);

        // The amount tokens that will be unstaked from the total to guarantee the "stake" share
        // price never decreases. The difference between `receive_amount` and `unstake_amount` is
        // paid from the allocated STAKE_SHARE_PRICE_GUARANTEE_FUND.
        let unstake_amount = self.staked_amount_from_num_shares_rounded_down(num_shares);

        self.total_staked_near_amount -= unstake_amount;
        self.total_share_amount -= num_shares;

        // Increase requested unstake amount within the current epoch
        self.epoch_requested_unstake_amount += unstake_amount;

        Event::Unstake {
            account_id: &account_id,
            unstaked_amount: &U128(receive_amount),
            burnt_stake_shares: &U128(num_shares),
            new_unstaked_balance: &U128(account.unstaked),
            new_stake_shares: &U128(account.stake_shares),
            unstaked_available_epoch_height: account.unstaked_available_epoch_height,
        }
        .emit();
        FtBurn {
            owner_id: &account_id,
            amount: &U128(num_shares),
            memo: Some("unstake"),
        }
        .emit();
        log!(
            "Contract total staked balance is {}. Total number of shares {}",
            self.total_staked_near_amount,
            self.total_share_amount
        );
    }

    /// Asserts that the method was called by the owner.
    pub(crate) fn assert_owner(&self) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            ERR_NOT_OWNER
        );
    }

    pub(crate) fn internal_get_beneficiaries(&self) -> HashMap<AccountId, u32> {
        let mut result: HashMap<AccountId, u32> = HashMap::new();
        for (account_id, bps) in self.beneficiaries.iter() {
            result.insert(account_id, bps);
        }

        result
    }

    /// When there are rewards, a part of them will be
    /// given to executor, manager or treasury by minting new LiNEAR tokens.
    pub(crate) fn internal_distribute_staking_rewards(&mut self, rewards: Balance) {
        let hashmap: HashMap<AccountId, u32> = self.internal_get_beneficiaries();
        for (account_id, bps) in hashmap.iter() {
            let reward_near_amount: Balance = bps_mul(rewards, *bps);
            // mint extra LiNEAR for him
            self.internal_mint_beneficiary_rewards(account_id, reward_near_amount);
        }
    }

    /// Mint new LiNEAR tokens to given account at the current price.
    /// This will DECREASE the LiNEAR price.
    fn internal_mint_beneficiary_rewards(
        &mut self,
        account_id: &AccountId,
        near_amount: Balance,
    ) -> ShareBalance {
        self.assert_running();

        let shares = self.num_shares_from_staked_amount_rounded_down(near_amount);
        // mint to account
        if self.accounts.get(account_id).is_none() {
            self.internal_register_account(account_id);
        }
        self.internal_ft_deposit(account_id, shares);
        FtMint {
            owner_id: account_id,
            amount: &U128(shares),
            memo: Some("beneficiary rewards"),
        }
        .emit();
        shares
    }

    /// Returns the number of "stake" shares rounded down corresponding to the given staked balance
    /// amount.
    ///
    /// price = total_staked / total_shares
    /// Price is fixed
    /// (total_staked + amount) / (total_shares + num_shares) = total_staked / total_shares
    /// (total_staked + amount) * total_shares = total_staked * (total_shares + num_shares)
    /// amount * total_shares = total_staked * num_shares
    /// num_shares = amount * total_shares / total_staked
    pub(crate) fn num_shares_from_staked_amount_rounded_down(
        &self,
        amount: Balance,
    ) -> ShareBalance {
        require!(
            self.total_staked_near_amount > 0,
            ERR_NON_POSITIVE_TOTAL_STAKED_BALANCE
        );
        (U256::from(self.total_share_amount) * U256::from(amount)
            / U256::from(self.total_staked_near_amount))
        .as_u128()
    }

    /// Returns the number of "stake" shares rounded up corresponding to the given staked balance
    /// amount.
    ///
    /// Rounding up division of `a / b` is done using `(a + b - 1) / b`.
    pub(crate) fn num_shares_from_staked_amount_rounded_up(&self, amount: Balance) -> ShareBalance {
        require!(
            self.total_staked_near_amount > 0,
            ERR_NON_POSITIVE_TOTAL_STAKED_BALANCE
        );
        ((U256::from(self.total_share_amount) * U256::from(amount)
            + U256::from(self.total_staked_near_amount - 1))
            / U256::from(self.total_staked_near_amount))
        .as_u128()
    }

    /// Returns the staked amount rounded down corresponding to the given number of "stake" shares.
    pub(crate) fn staked_amount_from_num_shares_rounded_down(
        &self,
        num_shares: ShareBalance,
    ) -> Balance {
        require!(
            self.total_share_amount > 0,
            ERR_NON_POSITIVE_TOTAL_STAKE_SHARES
        );
        (U256::from(self.total_staked_near_amount) * U256::from(num_shares)
            / U256::from(self.total_share_amount))
        .as_u128()
    }

    /// Returns the staked amount rounded up corresponding to the given number of "stake" shares.
    ///
    /// Rounding up division of `a / b` is done using `(a + b - 1) / b`.
    pub(crate) fn staked_amount_from_num_shares_rounded_up(
        &self,
        num_shares: ShareBalance,
    ) -> Balance {
        require!(
            self.total_share_amount > 0,
            ERR_NON_POSITIVE_TOTAL_STAKE_SHARES
        );
        ((U256::from(self.total_staked_near_amount) * U256::from(num_shares)
            + U256::from(self.total_share_amount - 1))
            / U256::from(self.total_share_amount))
        .as_u128()
    }

    /// Inner method to get the given account or a new default value account.
    pub(crate) fn internal_get_account(&self, account_id: &AccountId) -> Account {
        self.accounts.get(account_id).unwrap_or_default()
    }

    /// Inner method to save the given account for a given account ID.
    pub(crate) fn internal_save_account(&mut self, account_id: &AccountId, account: &Account) {
        self.accounts.insert(account_id, account);
    }
}

// -- manager related methods
impl LiquidStakingContract {
    pub(crate) fn internal_add_manager(&mut self, manager_id: &AccountId) {
        self.assert_running();
        self.managers.insert(manager_id);
    }

    pub(crate) fn internal_remove_manager(&mut self, manager_id: &AccountId) -> bool {
        self.assert_running();
        self.managers.remove(manager_id)
    }

    pub(crate) fn internal_get_managers(&self) -> Vec<AccountId> {
        self.managers.to_vec()
    }

    pub(crate) fn assert_manager(&self) {
        require!(
            self.managers.contains(&env::predecessor_account_id()),
            ERR_NOT_MANAGER
        );
    }

    pub(crate) fn signed_by_manager(&self) -> bool {
        self.managers.contains(&env::signer_account_id())
    }
}
