import { BN, Gas, NEAR, NearAccount } from 'near-workspaces';
import {
  assertFailure,
  initWorkSpace,
  createStakingPool,
  updateBaseStakeAmounts,
  setManager,
  assertValidatorAmountHelper,
  epochStake,
  epochUnstake,
  amountWithDiff,
  test,
} from './helper';

async function stakeAll(owner: NearAccount, contract: NearAccount) {
  let run = true;
  while (run) {
    run = await epochStake(owner, contract);
  }
}

async function unstakeAll(owner: NearAccount, contract: NearAccount) {
  let run = true;
  while (run) {
    run = await epochUnstake(owner, contract);
  }
}

test.before(async (t) => {
  t.context = await initWorkSpace();
});

test.after(async (t) => {
  await t.context.worker.tearDown();
});

test(
  'epoch stake',
  async (t) => {
    const { root, contract, alice, owner, bob } = t.context;
    const assertValidator = assertValidatorAmountHelper(t, contract, owner);

    const v1 = await createStakingPool(root, 'v1');
    const v2 = await createStakingPool(root, 'v2');
    const v3 = await createStakingPool(root, 'v3');

    // add validators to contract
    // weights:
    // - v1: 10
    // - v2: 20
    // - v3: 30
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
        weight: 20,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
    );
    await owner.call(
      contract,
      'add_validator',
      {
        validator_id: v3.accountId,
        weight: 30,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
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

    // at this time there should be no NEAR actually staked on validators
    await assertValidator(v1, '0', '0');
    await assertValidator(v2, '0', '0');
    await assertValidator(v3, '0', '0');

    // epoch stake
    await stakeAll(owner, contract);

    // validators should have staked balance based on their weights
    // note that 10 NEAR is already staked when contract init
    await assertValidator(v1, '10', '0');
    await assertValidator(v2, '20', '0');
    await assertValidator(v3, '30', '0');

    // fast-forward
    await owner.call(contract, 'set_epoch_height', { epoch: 11 });

    // stake more
    await bob.call(
      contract,
      'deposit_and_stake',
      {},
      {
        attachedDeposit: NEAR.parse('90'),
      },
    );

    // epoch stake
    await stakeAll(owner, contract);

    // validators should have staked balance based on their weights
    // note that 10 NEAR is already staked when contract init
    await assertValidator(v1, `${10 + 15}`, '0');
    await assertValidator(v2, `${20 + 30}`, '0');
    await assertValidator(v3, `${30 + 45}`, '0');

    // ---- Test base stake amount ----

    // set manager
    const manager = await setManager(root, contract, owner);

    // update base stake amount
    await updateBaseStakeAmounts(
      contract,
      manager,
      [v1.accountId],
      [NEAR.parse('20')],
    );

    // fast-forward
    await owner.call(contract, 'set_epoch_height', { epoch: 12 });

    // stake more
    await bob.call(
      contract,
      'deposit_and_stake',
      {},
      {
        attachedDeposit: NEAR.parse('50'),
      },
    );

    // epoch stake
    await stakeAll(owner, contract);

    // validators should have staked balance based on their weights + base stake amounts
    await assertValidator(v1, `${10 + 15 + 25}`, '0', '20');
    await assertValidator(v2, `${20 + 30 + 10}`, '0', '0');
    await assertValidator(v3, `${30 + 45 + 15}`, '0', '0');
  },
);

test(
  'epoch stake, staking pool with 1yN rounding diff',
  async (t) => {
    const { root, contract, alice, owner, bob } = t.context;
    const assertValidator = assertValidatorAmountHelper(t, contract, owner);

    const v1 = await createStakingPool(root, 'v1');
    const v2 = await createStakingPool(root, 'v2');
    const v3 = await createStakingPool(root, 'v3');

    // 1 yN rounding diff from staking pool contract
    const diff = NEAR.from(1);
    await owner.call(v1, 'set_balance_delta', {
      staked_delta: diff.toString(10),
      unstaked_delta: diff.toString(10),
    });

    // add validators to contract
    // weights:
    // - v1: 10
    // - v2: 20
    // - v3: 30
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
        weight: 20,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
    );
    await owner.call(
      contract,
      'add_validator',
      {
        validator_id: v3.accountId,
        weight: 30,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
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

    // at this time there should be no NEAR actually staked on validators
    await assertValidator(v1, '0', '0');
    await assertValidator(v2, '0', '0');
    await assertValidator(v3, '0', '0');

    // epoch stake
    await stakeAll(owner, contract);

    // validators should have staked balance based on their weights
    // note that 10 NEAR is already staked when contract init
    await assertValidator(
      v1,
      amountWithDiff('10', diff, -1),
      amountWithDiff('0', diff, 1),
    );
    await assertValidator(v2, '20', '0');
    await assertValidator(v3, '30', '0');

    // fast-forward
    await owner.call(contract, 'set_epoch_height', { epoch: 11 });

    // stake more
    await bob.call(
      contract,
      'deposit_and_stake',
      {},
      {
        attachedDeposit: NEAR.parse('90'),
      },
    );

    // epoch stake
    await stakeAll(owner, contract);

    // validators should have staked balance based on their weights
    // note that 10 NEAR is already staked when contract init
    await assertValidator(
      v1,
      amountWithDiff(`${10 + 15}`, diff, -2),
      amountWithDiff('0', diff, 2),
    );
    await assertValidator(v2, `${20 + 30}`, '0');
    await assertValidator(v3, `${30 + 45}`, '0');

    // ---- Test base stake amount ----

    // set manager
    const manager = await setManager(root, contract, owner);

    // update base stake amount
    await updateBaseStakeAmounts(
      contract,
      manager,
      [v1.accountId],
      [NEAR.parse('20')],
    );

    // fast-forward
    await owner.call(contract, 'set_epoch_height', { epoch: 12 });

    // stake more
    await bob.call(
      contract,
      'deposit_and_stake',
      {},
      {
        attachedDeposit: NEAR.parse('50'),
      },
    );

    // epoch stake
    await stakeAll(owner, contract);

    // validators should have staked balance based on their weights + base stake amounts
    // - v1 is selected first, and to meet the target amount, 25 N + 2 yN will be staked, which reduced the diff for staked amount
    // - v3 is then selected since its delta is higher than v2, though their delta/target are the same
    // - v2 is finally selected with 2 yN diff which is moved to v1
    await assertValidator(
      v1,
      amountWithDiff(`${10 + 15 + 25}`, diff, -1),
      amountWithDiff('0', diff, 3),
      '20',
    );
    await assertValidator(
      v2,
      amountWithDiff(`${20 + 30 + 10}`, diff, -2),
      '0',
      '0',
    );
    await assertValidator(v3, `${30 + 45 + 15}`, '0', '0');
  },
);

test(
  'epoch unstake',
  async (t) => {
    const { root, contract, alice, owner } = t.context;
    const assertValidator = assertValidatorAmountHelper(t, contract, owner);

    const v1 = await createStakingPool(root, 'v1');
    const v2 = await createStakingPool(root, 'v2');
    const v3 = await createStakingPool(root, 'v3');

    // add validators to contract
    // weights:
    // - v1: 10
    // - v2: 20
    // - v3: 30
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
        weight: 20,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
    );
    await owner.call(
      contract,
      'add_validator',
      {
        validator_id: v3.accountId,
        weight: 30,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
    );

    // user stake
    await alice.call(
      contract,
      'deposit_and_stake',
      {},
      {
        attachedDeposit: NEAR.parse('110'),
      },
    );

    // epoch stake
    await stakeAll(owner, contract);

    // fast-forward epoch
    await owner.call(contract, 'set_epoch_height', { epoch: 14 });

    // user unstake
    await alice.call(contract, 'unstake', { amount: NEAR.parse('30') });

    // at this time no actual unstake should happen
    await assertValidator(v1, '20', '0');
    await assertValidator(v2, '40', '0');
    await assertValidator(v3, '60', '0');

    // epoch unstake
    await unstakeAll(owner, contract);

    // 60 NEAR was initially staked, 30 was taken out
    await assertValidator(v1, '20', '0');
    await assertValidator(v2, '32.5', '7.5');
    await assertValidator(v3, '37.5', '22.5');

    // unstake more
    await alice.call(contract, 'unstake', { amount: NEAR.parse('18') });

    // epoch unstake should not take effect now
    await assertValidator(v1, '20', '0');
    await assertValidator(v2, '32.5', '7.5');
    await assertValidator(v3, '37.5', '22.5');

    // fast-forward
    await owner.call(contract, 'set_epoch_height', { epoch: 18 });

    await unstakeAll(owner, contract);

    await assertValidator(v1, '12', '8');
    await assertValidator(v2, '22.5', '17.5');
    await assertValidator(v3, '37.5', '22.5');

    // ---- Test base stake amount ----

    // set manager
    const manager = await setManager(root, contract, owner);

    // update base stake amount of v1 to 10 NEAR
    await updateBaseStakeAmounts(
      contract,
      manager,
      [v1.accountId],
      [NEAR.parse('10')],
    );

    // fast-forward
    await owner.call(contract, 'set_epoch_height', { epoch: 22 });

    // unstake more; remaining total staked: 120 - 30 - 18 - 26 = 46
    await alice.call(contract, 'unstake', { amount: NEAR.parse('26') });

    // epoch unstake
    await unstakeAll(owner, contract);

    // validators should have target stake amount based on weights + base stake amounts
    // - 1st epoch_unstake() unstaked 19.5 NEAR (amount = delta) from validator v3
    // - 2nd epoch_unstake() unstaked 6.5 NEAR (amount = rest) from validator v2
    await assertValidator(v1, '12', '8', '10', '16'); // target = 10 (base) + 6 (weighted) = 16; delta (1st) = 12 - 16 = -4; delta (2nd) = 12 - 16 = -4;
    await assertValidator(v2, '16', '24', '0', '12'); // target = 12 (weighted); delta (1st) = 22.5 - 12 = 10.5; delta (2nd) = 16 - 12 = 4;
    await assertValidator(v3, '18', '42', '0', '18'); // target = 18 (weighted); delta (1st) = 37.5 - 18 = 19.5; delta (2nd) = 18 - 18 = 0;

    // reset base stake amount of v1 to 0
    await updateBaseStakeAmounts(
      contract,
      manager,
      [v1.accountId],
      [NEAR.parse('0')],
    );

    // fast-forward
    await owner.call(contract, 'set_epoch_height', { epoch: 26 });

    // unstake more; remaining total staked: 120 - 30 - 18 - 26 - 10 = 36
    await alice.call(contract, 'unstake', { amount: NEAR.parse('10') });

    // epoch unstake
    await unstakeAll(owner, contract);

    // validators should have target stake amount based on weights + base stake amounts
    // - 1st epoch_unstake() unstaked 6 NEAR (amount = delta) from validator v1;
    // - 2nd epoch_unstake() unstaked 4 NEAR (amount = rest) from validator v2;
    await assertValidator(v1, '6', '14', '0', '6'); // target = 6 (weighted); delta (1st) = 12 - 6 = 6; delta (2nd) = 6 - 6 = 0;
    await assertValidator(v2, '12', '28', '0', '12'); // target = 12 (weighted); delta (1st) = 16 - 12 = 4; delta (2nd) = 12 - 12 = 0;
    await assertValidator(v3, '18', '42', '0', '18'); // target = 18 (weighted); delta (1st) = 18 - 18 = 0; delta (2nd) = 18 - 18 = 0;
  },
);

test(
  'epoch unstake, staking pool with 1yN rounding diff',
  async (t) => {
    const { root, contract, alice, owner } = t.context;
    const assertValidator = assertValidatorAmountHelper(t, contract, owner);

    const v1 = await createStakingPool(root, 'v1');
    const v2 = await createStakingPool(root, 'v2');
    const v3 = await createStakingPool(root, 'v3');

    // 1 yN rounding diff from staking pool contract
    const diff = NEAR.from(1);
    await owner.call(v1, 'set_balance_delta', {
      staked_delta: diff.toString(10),
      unstaked_delta: diff.toString(10),
    });

    // add validators to contract
    // weights:
    // - v1: 10
    // - v2: 20
    // - v3: 30
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
        weight: 20,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
    );
    await owner.call(
      contract,
      'add_validator',
      {
        validator_id: v3.accountId,
        weight: 30,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
    );

    // user stake
    await alice.call(
      contract,
      'deposit_and_stake',
      {},
      {
        attachedDeposit: NEAR.parse('110'),
      },
    );

    // epoch stake
    await stakeAll(owner, contract);

    // fast-forward epoch
    await owner.call(contract, 'set_epoch_height', { epoch: 14 });

    // user unstake
    await alice.call(contract, 'unstake', { amount: NEAR.parse('30') });

    // at this time no actual unstake should happen
    await assertValidator(
      v1,
      amountWithDiff('20', diff, -1),
      amountWithDiff('0', diff, 1),
    );
    await assertValidator(v2, '40', '0');
    await assertValidator(v3, '60', '0');

    // epoch unstake
    await unstakeAll(owner, contract);

    // 60 NEAR was initially staked, 30 was taken out
    await assertValidator(
      v1,
      amountWithDiff('20', diff, -1),
      amountWithDiff('0', diff, 1),
    );
    await assertValidator(v2, '32.5', '7.5');
    await assertValidator(v3, '37.5', '22.5');

    // unstake more
    await alice.call(contract, 'unstake', { amount: NEAR.parse('18') });

    // epoch unstake should not take effect now
    await assertValidator(
      v1,
      amountWithDiff('20', diff, -1),
      amountWithDiff('0', diff, 1),
    );
    await assertValidator(v2, '32.5', '7.5');
    await assertValidator(v3, '37.5', '22.5');

    // fast-forward
    await owner.call(contract, 'set_epoch_height', { epoch: 18 });

    await unstakeAll(owner, contract);

    await assertValidator(
      v1,
      amountWithDiff('12', diff, -1),
      amountWithDiff('8', diff, 1),
    );
    await assertValidator(
      v2,
      amountWithDiff('22.5', diff, -1),
      amountWithDiff('17.5', diff, 1),
    );
    await assertValidator(v3, '37.5', '22.5');

    // ---- Test base stake amount ----

    // set manager
    const manager = await setManager(root, contract, owner);

    // update base stake amount of v1 to 10 NEAR
    await updateBaseStakeAmounts(
      contract,
      manager,
      [v1.accountId],
      [NEAR.parse('10')],
    );

    // fast-forward
    await owner.call(contract, 'set_epoch_height', { epoch: 22 });

    // unstake more; remaining total staked: 120 - 30 - 18 - 26 = 46
    await alice.call(contract, 'unstake', { amount: NEAR.parse('26') });

    // epoch unstake
    await unstakeAll(owner, contract);

    // validators should have target stake amount based on weights + base stake amounts
    // - 1st epoch_unstake() unstaked 19.5 NEAR (amount = delta) from validator v3
    // - 2nd epoch_unstake() unstaked 6.5 NEAR (amount = rest) from validator v2
    await assertValidator(
      v1,
      amountWithDiff('12', diff, -1),
      amountWithDiff('8', diff, 1),
      '10',
      '16',
    ); // target = 10 (base) + 6 (weighted) = 16; delta (1st) = 12 - 16 = -4; delta (2nd) = 12 - 16 = -4;
    await assertValidator(
      v2,
      amountWithDiff('16', diff, -1),
      amountWithDiff('24', diff, 1),
      '0',
      '12',
    ); // target = 12 (weighted); delta (1st) = 22.5 - 12 = 10.5; delta (2nd) = 16 - 12 = 4;
    await assertValidator(v3, '18', '42', '0', '18'); // target = 18 (weighted); delta (1st) = 37.5 - 18 = 19.5; delta (2nd) = 18 - 18 = 0;

    // reset base stake amount of v1 to 0
    await updateBaseStakeAmounts(
      contract,
      manager,
      [v1.accountId],
      [NEAR.parse('0')],
    );

    // fast-forward
    await owner.call(contract, 'set_epoch_height', { epoch: 26 });

    // unstake more; remaining total staked: 120 - 30 - 18 - 26 - 10 = 36
    await alice.call(contract, 'unstake', { amount: NEAR.parse('10') });

    // epoch unstake
    await unstakeAll(owner, contract);

    // validators should have target stake amount based on weights + base stake amounts
    // - 1st epoch_unstake() unstaked 6 NEAR (amount = delta) from validator v1;
    // - 2nd epoch_unstake() unstaked 4 NEAR (amount = rest) from validator v2;
    await assertValidator(
      v1,
      amountWithDiff('6', diff, -1),
      amountWithDiff('14', diff, 1),
      '0',
      '6',
    ); // target = 6 (weighted); delta (1st) = 12 - 6 = 6; delta (2nd) = 6 - 6 = 0;
    await assertValidator(
      v2,
      amountWithDiff('12', diff, -2),
      amountWithDiff('28', diff, 2),
      '0',
      '12',
    ); // target = 12 (weighted); delta (1st) = 16 - 12 = 4; delta (2nd) = 12 - 12 = 0;
    await assertValidator(v3, '18', '42', '0', '18'); // target = 18 (weighted); delta (1st) = 18 - 18 = 0; delta (2nd) = 18 - 18 = 0;

    // fast-forward
    await owner.call(contract, 'set_epoch_height', { epoch: 30 });

    // reset v1 weight to 0
    await owner.call(contract, 'update_weight', {
      validator_id: v1.accountId,
      weight: 0,
    });

    // unstake more; remaining total staked: 120 - 30 - 18 - 26 - 10 - 6 = 30
    await alice.call(contract, 'unstake', {
      amount: NEAR.parse('6').sub(NEAR.from('3')),
    });

    // epoch unstake
    await unstakeAll(owner, contract);

    // validators should have target stake amount based on weights + base stake amounts
    // - 1st epoch_unstake() unstaked ~6 NEAR (amount = delta) from validator v1;
    await assertValidator(v1, '1 yN', amountWithDiff('20', diff, -1), '0', '0'); // target = 0 (weighted);
    await assertValidator(
      v2,
      amountWithDiff('12', diff, -2),
      amountWithDiff('28', diff, 2),
      '0',
      amountWithDiff('12', diff, 1),
    ); // target = 12 (weighted);
    await assertValidator(v3, '18', '42', '0', amountWithDiff('18', diff, 1)); // target = 18 (weighted);

    // fast-forward
    await owner.call(contract, 'set_epoch_height', { epoch: 34 });

    // unstake more; remaining total staked: 120 - 30 - 18 - 26 - 10 - 6 - 10 = 20
    await alice.call(contract, 'unstake', { amount: NEAR.parse('10') });

    // epoch unstake
    await unstakeAll(owner, contract);

    // validators should have target stake amount based on weights + base stake amounts
    // - 1st epoch_unstake() unstaked 1 yocto NEAR (amount = delta) from validator v1, because the target of v1 is 0;
    // - 2nd epoch_unstake() unstaked 6 NEAR (amount = delta) from validator v3, because the delta / target ratio of v3 is higher than v2;
    // - 3rd epoch_unstake() unstaked ~4 NEAR (amount = delta) from validator v2;
    await assertValidator(v1, '0', amountWithDiff('20', diff, 1), '0', '0'); // target = 0 (weighted);
    await assertValidator(
      v2,
      amountWithDiff('8', diff, -1),
      amountWithDiff('32', diff, 1),
      '0',
      amountWithDiff('8', diff, 1),
    ); // target = 8 (weighted);
    await assertValidator(v3, '12', '48', '0', amountWithDiff('12', diff, 1)); // target = 12 (weighted);
  },
);

test(
  'epoch collect rewards',
  async (t) => {
    const { root, contract, alice, owner } = t.context;
    t.timeout(60 * 1000);
    const assertValidator = assertValidatorAmountHelper(t, contract, owner);

    const v1 = await createStakingPool(root, 'v1');
    const v2 = await createStakingPool(root, 'v2');
    const v3 = await createStakingPool(root, 'v3');

    // add validators to contract
    // weights:
    // - v1: 10
    // - v2: 20
    // - v3: 30
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
        weight: 20,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
    );
    await owner.call(
      contract,
      'add_validator',
      {
        validator_id: v3.accountId,
        weight: 30,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
    );

    // set manager
    const manager = await setManager(root, contract, owner);

    // update base stake amount of v1 to 10 NEAR
    await updateBaseStakeAmounts(
      contract,
      manager,
      [v1.accountId],
      [NEAR.parse('10')],
    );

    // user stake
    await alice.call(
      contract,
      'deposit_and_stake',
      {},
      {
        attachedDeposit: NEAR.parse('60'),
      },
    );

    // epoch stake
    await stakeAll(owner, contract);

    let total_share_amount_0 = NEAR.from(
      await contract.view('get_total_share_amount'),
    );
    let total_near_amount_0 = NEAR.from(
      await contract.view('get_total_staked_balance'),
    );
    t.truthy(total_share_amount_0.eq(NEAR.parse('70')));
    t.truthy(total_near_amount_0.eq(NEAR.parse('70')));

    // generate rewards
    await contract.call(v1, 'add_reward', {
      amount: NEAR.parse('2').toString(),
    });
    await contract.call(v2, 'add_reward', {
      amount: NEAR.parse('2').toString(),
    });
    await contract.call(v3, 'add_reward', {
      amount: NEAR.parse('3').toString(),
    });

    // update rewards
    await owner.call(
      contract,
      'epoch_update_rewards',
      {
        validator_id: v1.accountId,
      },
      {
        gas: Gas.parse('200 Tgas'),
      },
    );
    await owner.call(
      contract,
      'epoch_update_rewards',
      {
        validator_id: v2.accountId,
      },
      {
        gas: Gas.parse('200 Tgas'),
      },
    );
    await owner.call(
      contract,
      'epoch_update_rewards',
      {
        validator_id: v3.accountId,
      },
      {
        gas: Gas.parse('200 Tgas'),
      },
    );

    let total_share_amount_1 = NEAR.from(
      await contract.view('get_total_share_amount'),
    );
    let total_near_amount_1 = NEAR.from(
      await contract.view('get_total_staked_balance'),
    );
    t.truthy(total_share_amount_1.eq(NEAR.parse('70')));
    t.truthy(total_near_amount_1.eq(NEAR.parse('77')));

    // check staked amount and base stake amount on each validator
    await assertValidator(v1, '22', '0', '11');
    await assertValidator(v2, '22', '0', '0');
    await assertValidator(v3, '33', '0', '0');

    // set beneficiary
    await owner.call(contract, 'set_beneficiary', {
      account_id: owner.accountId,
      bps: 1000,
    });

    // generate more rewards
    await contract.call(v1, 'add_reward', {
      amount: NEAR.parse('2').toString(),
    });

    await owner.call(
      contract,
      'epoch_update_rewards',
      {
        validator_id: v1.accountId,
      },
      {
        gas: Gas.parse('200 Tgas'),
      },
    );

    let total_share_amount_2 = NEAR.from(
      await contract.view('get_total_share_amount'),
    );
    let total_near_amount_2 = NEAR.from(
      await contract.view('get_total_staked_balance'),
    );
    t.is(total_share_amount_2.toString(), '70177215189873417721518987');
    t.is(total_near_amount_2.toString(), '79000000000000000000000000');

    // check staked amount and base stake amount on each validator
    await assertValidator(v1, '24', '0', '12');
    await assertValidator(v2, '22', '0', '0');
    await assertValidator(v3, '33', '0', '0');

    // fast-forward
    await owner.call(contract, 'set_epoch_height', { epoch: 14 });

    const aliceBalance = await contract.view('get_account_total_balance', {
      account_id: alice,
    });

    // unstake 69 NEAR; remaining total staked: 79 - 67.5 = 11.5
    await alice.call(contract, 'unstake', { amount: NEAR.parse('67.5') });

    // epoch unstake
    await unstakeAll(owner, contract);

    // check staked amount and base stake amount on each validator
    // There're 1yN diff due to rounding when alice unstakes 67.5 NEAR
    const diff = NEAR.from(1);
    await assertValidator(
      v1,
      amountWithDiff('11.5', diff, -1),
      amountWithDiff('12.5', diff, 1),
      '12',
    );
    await assertValidator(v2, '0', '22', '0');
    await assertValidator(v3, '0', '33', '0');

    // fast-forward 4 epoch
    await owner.call(contract, 'set_epoch_height', { epoch: 18 });

    // withdraw again
    await owner.call(
      contract,
      'epoch_withdraw',
      {
        validator_id: v2.accountId,
      },
      {
        gas: Gas.parse('200 Tgas'),
      },
    );

    // check staked amount and base stake amount on each validator
    await assertValidator(
      v1,
      amountWithDiff('11.5', diff, -1),
      amountWithDiff('12.5', diff, 1),
      '12',
    );
    await assertValidator(v2, '0', '0', '0');
    await assertValidator(v3, '0', '33', '0');

    // generate more rewards
    await contract.call(v2, 'add_reward', {
      amount: NEAR.parse('2').toString(),
    });

    // update rewards for validator with 0 staked and unstaked amount
    await owner.call(
      contract,
      'epoch_update_rewards',
      {
        validator_id: v2.accountId,
      },
      {
        gas: Gas.parse('200 Tgas'),
      },
    );

    // check staked amount and base stake amount on each validator
    await assertValidator(
      v1,
      amountWithDiff('11.5', diff, -1),
      amountWithDiff('12.5', diff, 1),
      '12',
    );
    await assertValidator(v2, '2', '0', '0');
    await assertValidator(v3, '0', '33', '0');
  },
);

test(
  'epoch withdraw',
  async (t) => {
    const { contract, alice, root, owner } = t.context;
    const assertValidator = assertValidatorAmountHelper(t, contract, owner);

    const v1 = await createStakingPool(root, 'v1');
    const v2 = await createStakingPool(root, 'v2');
    const v3 = await createStakingPool(root, 'v3');

    // add validators to contract
    // weights:
    // - v1: 10
    // - v2: 20
    // - v3: 30
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
        weight: 20,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
    );
    await owner.call(
      contract,
      'add_validator',
      {
        validator_id: v3.accountId,
        weight: 30,
      },
      {
        gas: Gas.parse('100 Tgas'),
      },
    );

    // user stake
    await alice.call(
      contract,
      'deposit_and_stake',
      {},
      {
        attachedDeposit: NEAR.parse('110'),
      },
    );

    // epoch stake
    await stakeAll(owner, contract);

    // fast-forward
    await owner.call(contract, 'set_epoch_height', { epoch: 11 });

    // user unstake
    await alice.call(contract, 'unstake', { amount: NEAR.parse('30') });

    // epoch unstake
    await unstakeAll(owner, contract);

    await assertValidator(v1, '20', '0');
    await assertValidator(v2, '32.5', '7.5');
    await assertValidator(v3, '37.5', '22.5');

    // withdraw should fail now
    await assertFailure(
      t,
      owner.call(
        contract,
        'epoch_withdraw',
        {
          validator_id: v2.accountId,
        },
        {
          gas: Gas.parse('200 Tgas'),
        },
      ),
      'Cannot withdraw from a pending release validator',
    );

    // withdraw should fail now
    await assertFailure(
      t,
      owner.call(
        contract,
        'epoch_withdraw',
        {
          validator_id: v3.accountId,
        },
        {
          gas: Gas.parse('200 Tgas'),
        },
      ),
      'Cannot withdraw from a pending release validator',
    );

    // fast-forward 4 epoch
    await owner.call(contract, 'set_epoch_height', { epoch: 15 });

    // withdraw again
    await owner.call(
      contract,
      'epoch_withdraw',
      {
        validator_id: v2.accountId,
      },
      {
        gas: Gas.parse('200 Tgas'),
      },
    );

    // withdraw again
    await owner.call(
      contract,
      'epoch_withdraw',
      {
        validator_id: v3.accountId,
      },
      {
        gas: Gas.parse('200 Tgas'),
      },
    );

    await assertValidator(v1, '20', '0');
    await assertValidator(v2, '32.5', '0');
    await assertValidator(v3, '37.5', '0');
  },
);

test.skip('estimate gas of epoch unstake', async (t) => {
  const {
    contract,
    alice,
    root,
    owner,
  } = t.context;
  const validatorsNum = 255;

  const names = Array.from(
    { length: validatorsNum },
    (_, index) => `v${index + 1}`,
  );
  const weights = names.map((_, index) =>
    Math.floor((index * 51) / validatorsNum),
  );

  const shuffleArray = (array: any[]) => {
    for (let i = array.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [array[i], array[j]] = [array[j], array[i]];
    }
  };

  shuffleArray(weights);

  const validators: NearAccount[] = [];
  for (const name of names) {
    validators.push(await createStakingPool(root, name));
  }

  let sliceIndex = 0;
  const sliceSize = 6;
  while (sliceIndex < validators.length) {
    const validatorIdSlice = validators
      .slice(sliceIndex, sliceSize)
      .map((v) => v.accountId);
    const weightSlice = weights.slice(sliceIndex, sliceSize);
    await owner.call(
      contract,
      'add_validators',
      {
        validator_ids: validatorIdSlice,
        weights: weightSlice,
      },
      {
        gas: Gas.parse('300 Tgas'),
      },
    );
    sliceIndex += sliceSize;
  }

  // user stake
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse('10000'),
    },
  );

  // epoch stake
  await stakeAll(owner, contract);

  // fast-forward
  await owner.call(contract, 'set_epoch_height', { epoch: 11 });

  // user unstake
  await alice.call(contract, 'unstake', { amount: NEAR.parse('2000') });

  // epoch unstake
  let run = true;
  const gasBurnts: Gas[] = [];
  while (run) {
    const outcome = await owner.callRaw(
      contract,
      'epoch_unstake',
      {},
      {
        gas: Gas.parse('280 Tgas'),
      },
    );

    const receiptsGasBurnt = outcome.result.receipts_outcome
      .map((receipt) => receipt.outcome.gas_burnt)
      .reduce((pre, gas) => pre + gas);
    gasBurnts.push(outcome.gas_burnt.add(new BN(receiptsGasBurnt)));
    const json = Buffer.from(outcome.SuccessValue!, 'base64').toString();
    run = JSON.parse(json);
  }
  console.log(gasBurnts.map((gas) => gas.toBigInt()));
});
