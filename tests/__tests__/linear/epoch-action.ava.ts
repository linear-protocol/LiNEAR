import { Gas, NEAR, NearAccount, stake, } from "near-workspaces-ava";
import { assertFailure, initWorkSpace, parseNEAR, skip } from "./helper";

const workspace = initWorkSpace();

async function createStakingPool (root: NearAccount, id: string) {
  return root.createAndDeploy(
    id,
    'compiled-contracts/mock_staking_pool.wasm',
    {
      method: 'new',
      args: {}
    }
  );
}

function assertValidatorAmountHelper (
  test: any,
  contract: NearAccount,
  owner: NearAccount
) {
  return async function (
    validator: NearAccount, 
    stakedAmount: string,
    unstakedAmount: string
  ) {
    // 1. make sure validator has correct balance
    test.is(
      await validator.view('get_account_staked_balance', { account_id: contract.accountId }),
      NEAR.parse(stakedAmount).toString()
    );
    test.is(
      await validator.view('get_account_unstaked_balance', { account_id: contract.accountId }),
      NEAR.parse(unstakedAmount).toString()
    );

    // 2. make sure contract validator object is synced
    const v: any = await owner.call(
      contract,
      'get_validator',
      {
        validator_id: validator.accountId
      }
    );
    const staked = parseNEAR(v.staked_amount);
    const unstaked = parseNEAR(v.unstaked_amount);
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

async function stakeAll (owner: NearAccount, contract: NearAccount) {
  let run = true;
  while (run) {
    run = await owner.call(
      contract,
      'epoch_stake',
      {},
      {
        gas: Gas.parse('200 Tgas')
      }
    );
  }
}

async function unstakeAll (owner: NearAccount, contract: NearAccount) {
  let run = true;
  while (run) {
    run = await owner.call(
      contract,
      'epoch_unstake',
      {},
      {
        gas: Gas.parse('200 Tgas')
      }
    );
  }
}

workspace.test('epoch stake', async (test, {root, contract, alice, owner, bob}) => {
  const assertValidator = assertValidatorAmountHelper(test, contract, owner);

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
      weight: 10
    }
  );
  await owner.call(
    contract,
    'add_validator',
    {
      validator_id: v2.accountId,
      weight: 20
    }
  );
  await owner.call(
    contract,
    'add_validator',
    {
      validator_id: v3.accountId,
      weight: 30
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
  await owner.call(
    contract,
    'set_epoch_height',
    { epoch: 11 }
  );

  // stake more
  await bob.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse('90')
    }
  );

  // epoch stake
  await stakeAll(owner, contract);

  // validators should have staked balance based on their weights
  // note that 10 NEAR is already staked when contract init
  await assertValidator(v1, `${10 + 15}`, '0');
  await assertValidator(v2, `${20 + 30}`, '0');
  await assertValidator(v3, `${30 + 45}`, '0');
});

workspace.test('epoch unstake', async (test, {root, contract, alice, owner}) => {
  const assertValidator = assertValidatorAmountHelper(test, contract, owner);

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
      weight: 10
    }
  );
  await owner.call(
    contract,
    'add_validator',
    {
      validator_id: v2.accountId,
      weight: 20
    }
  );
  await owner.call(
    contract,
    'add_validator',
    {
      validator_id: v3.accountId,
      weight: 30
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

  // epoch stake
  await stakeAll(owner, contract);

  // fast-forward epoch
  await owner.call(
    contract,
    'set_epoch_height',
    { epoch: 14 }
  );

  // user unstake
  await alice.call(
    contract,
    'unstake',
    { amount: NEAR.parse('30') }
  );

  // at this time no actual unstake should happen
  await assertValidator(v1, '10', '0');
  await assertValidator(v2, '20', '0');
  await assertValidator(v3, '30', '0');

  // epoch unstake
  await unstakeAll(owner, contract);

  // 60 NEAR was initially staked, 30 was taken out
  await assertValidator(v1, '10', '0');
  await assertValidator(v2, '20', '0');
  await assertValidator(v3, '0', '30');

  // unstake more
  await alice.call(
    contract,
    'unstake',
    { amount: NEAR.parse('18') }
  );

  // epoch unstake should not take effect now
  await assertValidator(v1, '10', '0');
  await assertValidator(v2, '20', '0');
  await assertValidator(v3, '0', '30');

  // fast-forward 
  await owner.call(
    contract,
    'set_epoch_height',
    { epoch: 18 }
  );

  // only 12 NEAR left in stake now
  await unstakeAll(owner, contract);
  await assertValidator(v1, '10', '0');
  await assertValidator(v2, '2', '18');
  await assertValidator(v3, '0', '30');
});

workspace.test('epoch collect rewards', async (test, {root, contract, alice, owner}) => {
  test.timeout(60 * 1000);

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
      weight: 10
    }
  );
  await owner.call(
    contract,
    'add_validator',
    {
      validator_id: v2.accountId,
      weight: 20
    }
  );
  await owner.call(
    contract,
    'add_validator',
    {
      validator_id: v3.accountId,
      weight: 30
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

  // epoch stake
  await stakeAll(owner, contract);

  let total_share_amount_0 = NEAR.from(await contract.view('get_total_share_amount'));
  let total_near_amount_0 = NEAR.from(await contract.view('get_total_staked_balance'));
  test.truthy(total_share_amount_0.eq(NEAR.parse('60')));
  test.truthy(total_near_amount_0.eq(NEAR.parse('60')));

  // generate rewards
  await contract.call(
    v1,
    'add_reward',
    { amount: NEAR.parse('1').toString() }
  );
  await contract.call(
    v2,
    'add_reward',
    { amount: NEAR.parse('2').toString() }
  );
  await contract.call(
    v3,
    'add_reward',
    { amount: NEAR.parse('3').toString() }
  );

  // update rewards
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
  await owner.call(
    contract,
    'epoch_update_rewards',
    {
      validator_id: v2.accountId
    },
    {
      gas: Gas.parse('200 Tgas')
    }
  );
  await owner.call(
    contract,
    'epoch_update_rewards',
    {
      validator_id: v3.accountId
    },
    {
      gas: Gas.parse('200 Tgas')
    }
  );

  let total_share_amount_1 = NEAR.from(await contract.view('get_total_share_amount'));
  let total_near_amount_1 = NEAR.from(await contract.view('get_total_staked_balance'));
  test.truthy(total_share_amount_1.eq(NEAR.parse('60')));
  test.truthy(total_near_amount_1.eq(NEAR.parse('66')));

  // set beneficiary
  await owner.call(
      contract,
      'set_beneficiary',
      {
          account_id: owner.accountId,
          fraction: {
              numerator: 1,
              denominator: 10
          }
      }
  );

  // generate more rewards
  await contract.call(
    v1,
    'add_reward',
    { amount: NEAR.parse('1').toString() }
  );

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

  let total_share_amount_2 = NEAR.from(await contract.view('get_total_share_amount'));
  let total_near_amount_2 = NEAR.from(await contract.view('get_total_staked_balance'));
  test.is(
    total_share_amount_2.toString(),
    '60089552238805970149253731'
  );
  test.is(
    total_near_amount_2.toString(),
    '67000000000000000000000000'
  );
});

workspace.test('epoch withdraw', async (test, {contract, alice, root, owner}) => {
  const assertValidator = assertValidatorAmountHelper(test, contract, owner);

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
      weight: 10
    }
  );
  await owner.call(
    contract,
    'add_validator',
    {
      validator_id: v2.accountId,
      weight: 20
    }
  );
  await owner.call(
    contract,
    'add_validator',
    {
      validator_id: v3.accountId,
      weight: 30
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

  // epoch stake
  await stakeAll(owner, contract);

  // fast-forward
  await owner.call(
    contract,
    'set_epoch_height',
    { epoch: 11 }
  );

  // user unstake
  await alice.call(
    contract,
    'unstake',
    { amount: NEAR.parse('30') }
  );

  // epoch unstake
  await unstakeAll(owner, contract);

  // withdraw should fail now
  await assertFailure(
    test,
    owner.call(
      contract,
      'epoch_withdraw',
      {
        validator_id: v3.accountId
      },
      {
        gas: Gas.parse('200 Tgas')
      }
    ),
    'Cannot withdraw from a pending release validator'
  );

  // fast-forward 4 epoch
  await owner.call(
    contract,
    'set_epoch_height',
    { epoch: 15 }
  );

  // withdraw again
  await owner.call(
    contract,
    'epoch_withdraw',
    {
      validator_id: v3.accountId
    },
    {
      gas: Gas.parse('200 Tgas')
    }
  );

  await assertValidator(v1, '10', '0');
  await assertValidator(v2, '20', '0');
  await assertValidator(v3, '0', '0');
});
