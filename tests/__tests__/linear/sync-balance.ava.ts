import {
  MANAGER_SYNC_BALANCE_DIFF_THRESHOLD,
  MAX_SYNC_BALANCE_DIFF,
  assertFailure,
  createStakingPool,
  epochStake,
  epochUnstake,
  getValidator,
  initWorkSpace,
  setManager,
  test,
} from './helper';
import { Gas, NEAR, NearAccount } from 'near-workspaces';

function assertValidatorAmountHelper(
  test: any,
  contract: NearAccount,
  owner: NearAccount,
) {
  return async function (
    validator: NearAccount,
    stakedAmount: string,
    unstakedAmount: string,
  ) {
    const v = await getValidator(contract, validator.accountId);
    const staked = NEAR.from(v.staked_amount);
    const unstaked = NEAR.from(v.unstaked_amount);
    test.is(staked.toString(), stakedAmount);
    test.is(unstaked.toString(), unstakedAmount);
  };
}

test.beforeEach(async (t) => {
  t.context = await initWorkSpace();
});

test.afterEach(async (t) => {
  await t.context.worker.tearDown();
});

test(
  'sync balance failure after stake/unstake',
  async (t) => {
    const { root, contract, alice, owner } = t.context;
    const assertValidator = assertValidatorAmountHelper(t, contract, owner);
    const v1 = await createStakingPool(root, 'v1');
    const v2 = await createStakingPool(root, 'v2');

    await owner.call(
      contract,
      'add_validator',
      {
        validator_id: v1.accountId,
        weight: 10,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
    );
    await owner.call(
      contract,
      'add_validator',
      {
        validator_id: v2.accountId,
        weight: 10,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
    );

    // 10 NEAR already in the contract
    await alice.call(
      contract,
      'deposit_and_stake',
      {},
      {
        attachedDeposit: NEAR.parse('50'),
      },
    );

    // -- 1. total balance diff > MAX_SYNC_BALANCE_DIFF
    const diff = MAX_SYNC_BALANCE_DIFF.addn(1);

    await owner.call(v1, 'set_balance_delta', {
      staked_delta: diff.toString(10),
      unstaked_delta: diff.toString(10),
    });

    for (let i = 0; i < 2; i++) {
      await epochStake(owner, contract);
    }

    // v1 amount should not change
    await assertValidator(v1, NEAR.parse('30').toString(10), '0');

    await owner.call(contract, 'set_epoch_height', { epoch: 11 });

    await alice.call(contract, 'unstake_all', {});

    for (let i = 0; i < 2; i++) {
      await epochUnstake(alice, contract);
    }

    // v2 amount should not change
    await assertValidator(
      v2,
      NEAR.parse('5').toString(10),
      NEAR.parse('25').toString(10),
    );
  },
);

test(
  'sync balance after stake/unstake',
  async (t) => {
    const { root, contract, alice, owner } = t.context;
    const assertValidator = assertValidatorAmountHelper(t, contract, owner);
    const v1 = await createStakingPool(root, 'v1');
    const v2 = await createStakingPool(root, 'v2');

    await owner.call(
      contract,
      'add_validator',
      {
        validator_id: v1.accountId,
        weight: 10,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
    );
    await owner.call(
      contract,
      'add_validator',
      {
        validator_id: v2.accountId,
        weight: 10,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
    );

    // 10 NEAR already in the contract
    await alice.call(
      contract,
      'deposit_and_stake',
      {},
      {
        attachedDeposit: NEAR.parse('50'),
      },
    );

    // -- amount balance diff < MAX_SYNC_BALANCE_DIFF
    const diff = MAX_SYNC_BALANCE_DIFF.subn(1);
    await owner.call(v2, 'set_balance_delta', {
      staked_delta: diff.toString(10),
      unstaked_delta: diff.toString(10),
    });

    for (let i = 0; i < 2; i++) {
      await epochStake(alice, contract);
    }

    await assertValidator(
      v2,
      NEAR.parse('30').sub(diff).toString(10),
      diff.toString(10),
    );

    await owner.call(contract, 'set_epoch_height', { epoch: 11 });

    await owner.call(v1, 'set_balance_delta', {
      staked_delta: diff.toString(10),
      unstaked_delta: diff.toString(10),
    });

    await alice.call(contract, 'unstake_all', {});

    for (let i = 0; i < 2; i++) {
      await epochUnstake(owner, contract);
    }

    await assertValidator(
      v1,
      NEAR.parse('5').sub(diff).toString(10),
      NEAR.parse('25').add(diff).toString(10),
    );
  },
);

test(
  'sync balance by manager failure',
  async (t) => {
    const { root, contract, alice, bob, owner } = t.context;
    // set bob as manager
    await setManager(root, contract, owner, bob);

    const assertValidator = assertValidatorAmountHelper(t, contract, owner);
    const v1 = await createStakingPool(root, 'v1');
    const v2 = await createStakingPool(root, 'v2');
    await owner.call(
      contract,
      'add_validator',
      {
        validator_id: v1.accountId,
        weight: 10,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
    );
    await owner.call(
      contract,
      'add_validator',
      {
        validator_id: v2.accountId,
        weight: 10,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
    );
    // 10 NEAR already in the contract
    await alice.call(
      contract,
      'deposit_and_stake',
      {},
      {
        attachedDeposit: NEAR.parse('50'),
      },
    );

    // -- 1. total balance diff > MANAGER_SYNC_BALANCE_DIFF_THRESHOLD
    const diff = MANAGER_SYNC_BALANCE_DIFF_THRESHOLD.addn(1);
    await owner.call(v1, 'set_balance_delta', {
      staked_delta: '0',
      unstaked_delta: diff.toString(10),
    });

    for (let i = 0; i < 2; i++) {
      await epochStake(owner, contract);
    }

    // sync balance only allowed by manager
    await assertFailure(
      t,
      alice.call(
        contract,
        'sync_balance_from_validator',
        {
          validator_id: v1.accountId,
        },
        {
          gas: Gas.parse('200 Tgas'),
        },
      ),
      'Only manager can perform this action',
    );

    await bob.call(
      contract,
      'sync_balance_from_validator',
      {
        validator_id: v1.accountId,
      },
      {
        gas: Gas.parse('200 Tgas'),
      },
    );

    // v1 amount should not change
    await assertValidator(v1, '30000000000000000000000000', '0');

    // -- 2. amount balance diff > MANAGER_SYNC_BALANCE_DIFF_THRESHOLD
    await owner.call(v2, 'set_balance_delta', {
      staked_delta: diff.toString(10),
      unstaked_delta: diff.toString(10),
    });

    await owner.call(contract, 'set_epoch_height', { epoch: 11 });

    await alice.call(contract, 'unstake_all', {});

    for (let i = 0; i < 2; i++) {
      await epochUnstake(alice, contract);
    }

    await bob.call(
      contract,
      'sync_balance_from_validator',
      {
        validator_id: v2.accountId,
      },
      {
        gas: Gas.parse('200 Tgas'),
      },
    );

    // v2 amount should not change
    await assertValidator(
      v2,
      NEAR.parse('5').toString(10),
      NEAR.parse('25').toString(10),
    );
  },
);

test(
  'sync balance by manager',
  async (t) => {
    const { root, contract, alice, bob, owner } = t.context;
    // set bob as manager
    await setManager(root, contract, owner, bob);

    const assertValidator = assertValidatorAmountHelper(t, contract, owner);
    const v1 = await createStakingPool(root, 'v1');
    const v2 = await createStakingPool(root, 'v2');
    await owner.call(
      contract,
      'add_validator',
      {
        validator_id: v1.accountId,
        weight: 10,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
    );
    await owner.call(
      contract,
      'add_validator',
      {
        validator_id: v2.accountId,
        weight: 10,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
    );
    // 10 NEAR already in the contract
    await alice.call(
      contract,
      'deposit_and_stake',
      {},
      {
        attachedDeposit: NEAR.parse('50'),
      },
    );

    // -- amount balance diff < MANAGER_SYNC_BALANCE_DIFF_THRESHOLD
    const diff = MANAGER_SYNC_BALANCE_DIFF_THRESHOLD.subn(1);
    await owner.call(v1, 'set_balance_delta', {
      staked_delta: diff.toString(10),
      unstaked_delta: diff.toString(10),
    });

    for (let i = 0; i < 2; i++) {
      await epochStake(alice, contract);
    }

    await bob.call(
      contract,
      'sync_balance_from_validator',
      {
        validator_id: v1.accountId,
      },
      {
        gas: Gas.parse('200 Tgas'),
      },
    );

    await assertValidator(
      v1,
      NEAR.parse('30').sub(diff).toString(10),
      diff.toString(10),
    );

    await owner.call(v2, 'set_balance_delta', {
      staked_delta: diff.toString(10),
      unstaked_delta: diff.toString(10),
    });

    await owner.call(contract, 'set_epoch_height', { epoch: 11 });

    await alice.call(contract, 'unstake_all', {});

    for (let i = 0; i < 2; i++) {
      await epochUnstake(alice, contract);
    }

    await bob.call(
      contract,
      'sync_balance_from_validator',
      {
        validator_id: v2.accountId,
      },
      {
        gas: Gas.parse('200 Tgas'),
      },
    );

    // v2 amount should not change
    await assertValidator(
      v2,
      NEAR.parse('5').sub(diff).toString(10),
      NEAR.parse('25').add(diff).toString(10),
    );
  },
);
