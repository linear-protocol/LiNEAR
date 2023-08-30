import { NearAccount, NEAR, Gas } from "near-workspaces-ava";
import { assertFailure, initWorkSpace, createStakingPool, getValidator } from "./helper";

const workspace = initWorkSpace();

async function setPanic(validator: NearAccount) {
  return validator.call(
    validator,
    'set_panic',
    {
      panic: true
    }
  );
}

function assertValidatorHelper(
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
      NEAR.parse(stakedAmount).toString()
    );
    test.is(
      unstaked.toString(),
      NEAR.parse(unstakedAmount).toString()
    );
  }
}

workspace.test('epoch stake failure: deposit_and_stake fails', async (test, { root, contract, owner, alice }) => {
  const assertValidator = assertValidatorHelper(test, contract, owner);

  const v1 = await createStakingPool(root, 'v1');

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

  // user stake
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse('50')
    }
  );

  await setPanic(v1);

  const ret = await owner.call(
    contract,
    'epoch_stake',
    {},
    {
      gas: Gas.parse('280 Tgas')
    }
  );

  test.is(ret, null);

  // nothing should be staked
  await assertValidator(v1, '0', '0');
});

workspace.test('epoch stake failure: get_account fails', async (test, { root, contract, owner, alice }) => {
  const assertValidator = assertValidatorHelper(test, contract, owner);

  const v1 = await createStakingPool(root, 'v1');

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

  // user stake
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse('50')
    }
  );

  v1.call(
    v1,
    'set_get_account_fail',
    {
      value: true
    }
  );

  const ret = await owner.call(
    contract,
    'epoch_stake',
    {},
    {
      gas: Gas.parse('280 Tgas')
    }
  );

  test.is(ret, null);

  // stake still succeeded
  await assertValidator(v1, '60', '0');
});

workspace.test('epoch stake failure: balance diff to large', async (test, { root, contract, owner, alice }) => {
  const assertValidator = assertValidatorHelper(test, contract, owner);

  const v1 = await createStakingPool(root, 'v1');

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

  // user stake
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse('50')
    }
  );

  const MAX_SYNC_BALANCE_DIFF = NEAR.from(100);
  const diff = MAX_SYNC_BALANCE_DIFF.addn(1);

  await owner.call(
    v1,
    'set_balance_delta',
    {
      staked_delta: diff.toString(10),
      unstaked_delta: diff.toString(10),
    },
  );

  const ret = await owner.call(
    contract,
    'epoch_stake',
    {},
    {
      gas: Gas.parse('280 Tgas')
    }
  );

  test.is(ret, null);

  // stake still succeeded
  await assertValidator(v1, '60', '0');
});

workspace.test('unstake failure', async (test, { root, contract, owner, alice }) => {
  const assertValidator = assertValidatorHelper(test, contract, owner);

  const v1 = await createStakingPool(root, 'v1');

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

  // user stake
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse('50')
    }
  );

  await owner.call(
    contract,
    'epoch_stake',
    {},
    {
      gas: Gas.parse('280 Tgas')
    }
  );

  await assertValidator(v1, '60', '0');

  // user unstake
  await alice.call(
    contract,
    'unstake',
    { amount: NEAR.parse('10') }
  );

  await setPanic(v1);

  await owner.call(
    contract,
    'epoch_unstake',
    {},
    {
      gas: Gas.parse('280 Tgas')
    }
  );

  // no unstake should actual happen
  await assertValidator(v1, '60', '0');
});

workspace.test('withdraw failure', async (test, { root, contract, owner, alice }) => {
  const assertValidator = assertValidatorHelper(test, contract, owner);

  const v1 = await createStakingPool(root, 'v1');

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

  // user stake
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse('50')
    }
  );

  await owner.call(
    contract,
    'epoch_stake',
    {},
    {
      gas: Gas.parse('280 Tgas')
    }
  );

  // fast-forward 4 epoch
  await owner.call(
    contract,
    'set_epoch_height',
    { epoch: 14 }
  );

  await assertValidator(v1, '60', '0');

  // user unstake
  await alice.call(
    contract,
    'unstake',
    { amount: NEAR.parse('10') }
  );

  await owner.call(
    contract,
    'epoch_unstake',
    {},
    {
      gas: Gas.parse('280 Tgas')
    }
  );

  await assertValidator(v1, '50', '10');

  // fast-forward
  await owner.call(
    contract,
    'set_epoch_height',
    { epoch: 18 }
  );

  await setPanic(v1);
  
  // withdraw
  await owner.call(
    contract,
    'epoch_withdraw',
    {
      validator_id: v1.accountId
    },
    {
      gas: Gas.parse('200 Tgas')
    }
  );

  // no actual withdraw should happen
  await assertValidator(v1, '50', '10');
});

workspace.test('get balance failure', async (test, { root, contract, owner, alice }) => {
  const assertValidator = assertValidatorHelper(test, contract, owner);

  const v1 = await createStakingPool(root, 'v1');

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

  // user stake
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse('50')
    }
  );

  await owner.call(
    contract,
    'epoch_stake',
    {},
    {
      gas: Gas.parse('280 Tgas')
    }
  );

  await assertValidator(v1, '60', '0');

  // generate rewards
  await contract.call(
    v1,
    'add_reward',
    { amount: NEAR.parse('1').toString() }
  );

  await setPanic(v1);

  // update reward
  await owner.call(
    contract,
    'epoch_update_rewards',
    {
      validator_id: v1.accountId
    },
    {
      gas: Gas.parse('200 Tgas')
    }
  );

  // balance should not change
  await assertValidator(v1, '60', '0');
});
