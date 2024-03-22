import { NearAccount, NEAR, Gas } from 'near-workspaces';
import {
  assertFailure,
  initWorkSpace,
  createStakingPool,
  setManager,
  assertValidatorAmountHelper,
  updateBaseStakeAmounts,
  getValidator,
  epochUnstake,
  epochStake,
  assertHasLog,
  test,
} from './helper';

async function stakeAll(signer: NearAccount, contract: NearAccount) {
  let run = true;
  while (run) {
    run = await epochStake(signer, contract);
  }
}

test.beforeEach(async (t) => {
  t.context = await initWorkSpace();
});

test.afterEach(async (t) => {
  await t.context.worker.tearDown();
});

test(
  'Non-manager call drain methods',
  async (t) => {
    const { contract, alice } = t.context
    await assertFailure(
      t,
      alice.call(
        contract,
        'drain_unstake',
        {
          validator_id: 'foo',
        },
        {
          gas: '275 Tgas',
        },
      ),
      'Only manager can perform this action',
    );
  },
);

test(
  'drain constraints',
  async (t) => {
    const { contract, root, owner, alice, bob } = t.context;
    const manager = alice;
    await setManager(root, contract, owner, manager);

    const v1 = await createStakingPool(root, 'v1');
    // add validator
    await manager.call(
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

    // update base stake amount to 20 NEAR
    await updateBaseStakeAmounts(
      contract,
      manager,
      [v1.accountId],
      [NEAR.parse('20')],
    );

    // user stake
    await alice.call(
      contract,
      'deposit_and_stake',
      {},
      {
        attachedDeposit: NEAR.parse('50'),
      },
    );

    // run stake
    await epochStake(bob, contract);

    // 1. cannot drain unstake when weight > 0
    await assertFailure(
      t,
      manager.call(
        contract,
        'drain_unstake',
        {
          validator_id: v1.accountId,
        },
        {
          gas: Gas.parse('275 Tgas'),
        },
      ),
      'Validator weight must be zero for drain operation',
    );

    // set weight to 0
    await manager.call(contract, 'update_weight', {
      validator_id: v1.accountId,
      weight: 0,
    });

    // 2. cannot drain unstake when base stake amount > 0
    await assertFailure(
      t,
      manager.call(
        contract,
        'drain_unstake',
        {
          validator_id: v1.accountId,
        },
        {
          gas: Gas.parse('275 Tgas'),
        },
      ),
      'Validator base stake amount must be zero for drain operation',
    );

    // update base stake amount to 0 NEAR
    await updateBaseStakeAmounts(
      contract,
      manager,
      [v1.accountId],
      [NEAR.parse('0')],
    );

    // fast-forward
    await owner.call(contract, 'set_epoch_height', { epoch: 11 });

    // user unstake
    await alice.call(contract, 'unstake_all', {});

    await epochUnstake(bob, contract);

    // validator now have unstaked balance > 0
    const assertValidator = assertValidatorAmountHelper(t, contract, owner);
    await assertValidator(v1, '10', '50');

    // -- 3. cannot drain unstake when pending release
    await assertFailure(
      t,
      manager.call(
        contract,
        'drain_unstake',
        {
          validator_id: v1.accountId,
        },
        {
          gas: Gas.parse('275 Tgas'),
        },
      ),
      'Cannot unstake from a pending release validator',
    );

    // fast-forward
    await owner.call(contract, 'set_epoch_height', { epoch: 15 });

    // -- 4. cannot drain unstake when unstaked balance > 0
    await assertFailure(
      t,
      manager.call(
        contract,
        'drain_unstake',
        {
          validator_id: v1.accountId,
        },
        {
          gas: Gas.parse('275 Tgas'),
        },
      ),
      'Validator unstaked amount too large for drain unstake',
    );
  },
);

test(
  'drain unstake and withdraw',
  async (t) => {
    const { contract, root, owner, alice, bob } = t.context;
    const manager = alice;
    await setManager(root, contract, owner, manager);

    const v1 = await createStakingPool(root, 'v1');
    const v2 = await createStakingPool(root, 'v2');

    // add validator
    await manager.call(
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
    await manager.call(
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

    // update base stake amount of v1 to 20 NEAR
    await updateBaseStakeAmounts(
      contract,
      manager,
      [v1.accountId],
      [NEAR.parse('20')],
    );

    // user stake
    await alice.call(
      contract,
      'deposit_and_stake',
      {},
      {
        attachedDeposit: NEAR.parse('50'),
      },
    );

    // run stake
    await stakeAll(bob, contract);

    /**
     * Steps to drain a validator
     * 1. set weight to 0
     * 2. set base stake amount to 0
     * 3. call drain_unstake
     * 4. call drain_withdraw
     */

    await manager.call(contract, 'update_weight', {
      validator_id: v1.accountId,
      weight: 0,
    });

    // reset base stake amount to 0 NEAR
    await updateBaseStakeAmounts(
      contract,
      manager,
      [v1.accountId],
      [NEAR.parse('0')],
    );

    await manager.call(
      contract,
      'drain_unstake',
      {
        validator_id: v1.accountId,
      },
      {
        gas: Gas.parse('275 Tgas'),
      },
    );

    // make sure the validator is in draining mode
    t.assert((await getValidator(contract, v1.accountId)).draining);

    // fast-forward
    await owner.call(contract, 'set_epoch_height', { epoch: 14 });

    // epoch_withdraw should not be allowed
    await assertFailure(
      t,
      manager.call(
        contract,
        'epoch_withdraw',
        {
          validator_id: v1.accountId,
        },
        {
          gas: Gas.parse('200 Tgas'),
        },
      ),
      'Validator is currently in draining process',
    );

    await manager.call(
      contract,
      'drain_withdraw',
      {
        validator_id: v1.accountId,
      },
      {
        gas: Gas.parse('200 Tgas'),
      },
    );

    // make sure v1 is drained
    t.assert(!(await getValidator(contract, v1.accountId)).draining);

    const assertValidator = assertValidatorAmountHelper(t, contract, owner);
    await assertValidator(v1, '0', '0');
    await assertValidator(v2, '20', '0');

    // restake and make sure funds are re-distributed
    await stakeAll(bob, contract);

    await assertValidator(v1, '0', '0');
    await assertValidator(v2, '60', '0');
  },
);

test(
  'drain unstake: get_account fails',
  async (t) => {
    const { contract, root, owner, alice, bob } = t.context;
    const manager = alice;
    await setManager(root, contract, owner, manager);

    const v1 = await createStakingPool(root, 'v1');
    const v2 = await createStakingPool(root, 'v2');

    // add validator
    await manager.call(
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
    await manager.call(
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

    // update base stake amount of v1 to 20 NEAR
    await updateBaseStakeAmounts(
      contract,
      manager,
      [v1.accountId],
      [NEAR.parse('20')],
    );

    // user stake
    await alice.call(
      contract,
      'deposit_and_stake',
      {},
      {
        attachedDeposit: NEAR.parse('50'),
      },
    );

    // run stake
    await stakeAll(bob, contract);

    /**
     * Steps to drain a validator
     * 1. set weight to 0
     * 2. set base stake amount to 0
     * 3. call drain_unstake
     * 4. call drain_withdraw
     */

    await manager.call(contract, 'update_weight', {
      validator_id: v1.accountId,
      weight: 0,
    });

    // reset base stake amount to 0 NEAR
    await updateBaseStakeAmounts(
      contract,
      manager,
      [v1.accountId],
      [NEAR.parse('0')],
    );

    v1.call(v1, 'set_get_account_fail', {
      value: true,
    });

    const ret = await manager.callRaw(
      contract,
      'drain_unstake',
      {
        validator_id: v1.accountId,
      },
      {
        gas: Gas.parse('275 Tgas'),
      },
    );

    assertHasLog(t, ret, 'sync_validator_balance_failed_cannot_get_account');
  },
);
