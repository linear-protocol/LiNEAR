import { assertFailure, createStakingPool, getValidator, initWorkSpace } from "./helper";
import { Gas, NEAR, NearAccount, ONE_NEAR, stake, } from "near-workspaces-ava";

const MAX_SYNC_BALANCE_DIFF = NEAR.from(100);

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
    const v = await getValidator(contract, validator.accountId);
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
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );
  await owner.call(
    contract,
    'add_validator',
    {
      validator_id: v2.accountId,
      weight: 10
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );

  // 10 NEAR already in the contract
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
        gas: Gas.parse('275 Tgas')
      }
    );
  }

  // -- 1. total balance diff > MAX_SYNC_BALANCE_DIFF
  const diff = MAX_SYNC_BALANCE_DIFF.addn(1);
  await owner.call(
    v1,
    'adjust_balance',
    {
      account_id: contract.accountId,
      staked_delta: "0",
      unstaked_delta: diff.toString(10)
    },
  );

  await owner.call(
    contract,
    'sync_balance_from_validator',
    {
      validator_id: v1.accountId
    },
    {
      gas: Gas.parse('200 Tgas')
    }
  );

  // v1 amount should not change
  await assertValidator(v1, '30000000000000000000000000', '0');

  // -- 2. amount balance diff > MAX_SYNC_BALANCE_DIFF
  await owner.call(
    v2,
    'adjust_balance',
    {
      account_id: contract.accountId,
      staked_delta: diff.toString(10),
      unstaked_delta: diff.toString(10)
    },
  );

  await owner.call(
    contract,
    'sync_balance_from_validator',
    {
      validator_id: v2.accountId
    },
    {
      gas: Gas.parse('200 Tgas')
    }
  );

  // v2 amount should not change
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
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );
  await owner.call(
    contract,
    'add_validator',
    {
      validator_id: v2.accountId,
      weight: 10
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );

  // 10 NEAR already in the contract
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
        gas: Gas.parse('275 Tgas')
      }
    );
  }

  // -- amount balance diff < MAX_SYNC_BALANCE_DIFF
  const diff = MAX_SYNC_BALANCE_DIFF.subn(1);
  await owner.call(
    v2,
    'adjust_balance',
    {
      account_id: contract.accountId,
      staked_delta: diff.toString(10),
      unstaked_delta: diff.toString(10),
    },
  );

  await owner.call(
    contract,
    'sync_balance_from_validator',
    {
      validator_id: v2.accountId
    },
    {
      gas: Gas.parse('200 Tgas')
    }
  );

  await assertValidator(v2, NEAR.parse("30").sub(diff).toString(10), diff.toString(10));
});
