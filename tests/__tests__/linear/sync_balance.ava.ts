import { assertFailure, createStakingPool, initWorkSpace } from "./helper";
import { Gas, NEAR, NearAccount, stake, } from "near-workspaces-ava";

const workspace = initWorkSpace();

function assertValidatorAmountHelper(
  test: any,
  contract: NearAccount,
  owner: NearAccount
) {
  return async function (
    validator: NearAccount,
    stakedAmount: string,
    unstakedAmount: string
  ) {
    const v: any = await owner.call(
      contract,
      'get_validator',
      {
        validator_id: validator.accountId
      }
    );
    const staked = NEAR.from(v.staked_amount);
    const unstaked = NEAR.from(v.unstaked_amount);
    test.is(
      staked.toString(),
      stakedAmount
    );
    test.is(
      unstaked.toString(),
      unstakedAmount
    );
  }
}

workspace.test('sync balance failure', async (test, { root, contract, alice, owner }) => {
  const assertValidator = assertValidatorAmountHelper(test, contract, owner);
  const v1 = await createStakingPool(root, 'v1');
  const v2 = await createStakingPool(root, 'v2');

  await owner.call(
    contract,
    'add_validator',
    {
      validator_id: v1.accountId,
      weight: 10
    }
  );
  await owner.call(
    contract,
    'add_validator',
    {
      validator_id: v2.accountId,
      weight: 10
    }
  );

  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse('50')
    }
  );

  for (let i = 0; i < 2; i++) {
    await owner.call(
      contract,
      'epoch_stake',
      {},
      {
        gas: Gas.parse('200 Tgas')
      }
    );
  }

  // -- 1. total balance diff > 1 yN
  await owner.call(
    v1,
    'adjust_balance',
    {
      account_id: contract.accountId,
      staked_delta: 1,
      unstaked_delta: 3
    },
  );

  await owner.call(
    contract,
    'sync_account_balance',
    {
      validator_id: v1.accountId
    },
    {
      gas: Gas.parse('200 Tgas')
    }
  );

  // v1 amount should not change
  await assertValidator(v1, '30000000000000000000000000', '0');

  // -- 2. amount balance diff > 100 yN
  await owner.call(
    v2,
    'adjust_balance',
    {
      account_id: contract.accountId,
      staked_delta: 101,
      unstaked_delta: 101
    },
  );

  await owner.call(
    contract,
    'sync_account_balance',
    {
      validator_id: v2.accountId
    },
    {
      gas: Gas.parse('200 Tgas')
    }
  );

  // v1 amount should not change
  await assertValidator(v2, '30000000000000000000000000', '0');
});

workspace.test('sync balance', async (test, { root, contract, alice, owner }) => {
  const assertValidator = assertValidatorAmountHelper(test, contract, owner);
  const v1 = await createStakingPool(root, 'v1');
  const v2 = await createStakingPool(root, 'v2');

  await owner.call(
    contract,
    'add_validator',
    {
      validator_id: v1.accountId,
      weight: 10
    }
  );
  await owner.call(
    contract,
    'add_validator',
    {
      validator_id: v2.accountId,
      weight: 10
    }
  );

  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse('50')
    }
  );

  for (let i = 0; i < 2; i++) {
    await owner.call(
      contract,
      'epoch_stake',
      {},
      {
        gas: Gas.parse('200 Tgas')
      }
    );
  }

  // -- amount balance diff < 100 yN
  await owner.call(
    v2,
    'adjust_balance',
    {
      account_id: contract.accountId,
      staked_delta: 99,
      unstaked_delta: 99
    },
  );

  await owner.call(
    contract,
    'sync_account_balance',
    {
      validator_id: v2.accountId
    },
    {
      gas: Gas.parse('200 Tgas')
    }
  );

  // v1 amount should not change
  await assertValidator(v2, '29999999999999999999999901', '99');
});
