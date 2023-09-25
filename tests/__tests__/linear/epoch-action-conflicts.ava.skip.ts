import { Gas, NEAR, NearAccount, stake, } from "near-workspaces-ava";
import {
  assertFailure,
  initWorkSpace,
  createStakingPool,
  updateBaseStakeAmounts,
  setManager,
  assertValidatorAmountHelper,
  getSummary,
  skip
} from "./helper";

const workspace = initWorkSpace();

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

skip('epoch stake', async (test, {root, contract, alice, owner, bob}) => {
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
      weight: 20
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );
  await owner.call(
    contract,
    'add_validator',
    {
      validator_id: v3.accountId,
      weight: 30
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


  // ---- Test base stake amount ----

  // set manager
  const manager = await setManager(root, contract, owner);

  // update base stake amount
  await updateBaseStakeAmounts(
    contract,
    manager,
    [
      v1.accountId,
    ],
    [
      NEAR.parse("20")
    ]
  );

  // fast-forward
  await owner.call(
    contract,
    'set_epoch_height',
    { epoch: 12 }
  );

  // stake more
  await bob.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse('50')
    }
  );

  // epoch stake
  await stakeAll(owner, contract);

  // validators should have staked balance based on their weights + base stake amounts
  await assertValidator(v1, `${10 + 15 + 25}`, '0', '20');
  await assertValidator(v2, `${20 + 30 + 10}`, '0', '0');
  await assertValidator(v3, `${30 + 45 + 15}`, '0', '0');
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
      weight: 20
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );
  await owner.call(
    contract,
    'add_validator',
    {
      validator_id: v3.accountId,
      weight: 30
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
      attachedDeposit: NEAR.parse('110')
    }
  );

  test.is(
    await contract.view("get_account_staked_balance", {
      account_id: alice
    }),
    NEAR.parse('110').toString()
  );
  test.is(
    await contract.view("get_account_unstaked_balance", {
      account_id: alice
    }),
    NEAR.parse('0').toString()
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

  test.is(
    await contract.view("get_account_staked_balance", {
      account_id: alice
    }),
    NEAR.parse('80').toString()
  );
  test.is(
    await contract.view("get_account_unstaked_balance", {
      account_id: alice
    }),
    NEAR.parse('30').toString()
  );

  // at this time no actual unstake should happen
  await assertValidator(v1, '20', '0');
  await assertValidator(v2, '40', '0');
  await assertValidator(v3, '60', '0');

  // epoch unstake
  // await unstakeAll(owner, contract);

  function sleep(ms: number) {
    return new Promise( resolve => setTimeout(resolve, ms) );
  }

  async function delayedEpochUnstake(ms) {
    await sleep(ms);
    const res = await owner.call_raw(
      contract,
      'epoch_unstake',
      {},
      {
        gas: Gas.parse('200 Tgas')
      }
    );
    console.log("ID 1", res.result.transaction_outcome.id);
  }

  async function epochUpdateRewards() {
    const res = await alice.call_raw(
      contract,
      'epoch_update_rewards',
      {
        validator_id: v3.accountId
      },
      {
        gas: Gas.parse('200 Tgas')
      }
    );
    console.log("ID 2", res.result.transaction_outcome.id);
  }

  // call epoch_unstake and epoch_update_rewards
  await Promise.all([
    epochUpdateRewards(),
    delayedEpochUnstake(1000),
  ]);

  // 60 NEAR was initially staked, 30 was taken out
  await assertValidator(v1, '20', '0');
  await assertValidator(v2, '40', '0');
  await assertValidator(v3, '30', '30');

  test.is(
    await contract.view("get_account_staked_balance", {
      account_id: alice
    }),
    NEAR.parse('80').toString()
  );
  test.is(
    await contract.view("get_account_unstaked_balance", {
      account_id: alice
    }),
    NEAR.parse('30').toString()
  );

  // // unstake more
  // await alice.call(
  //   contract,
  //   'unstake',
  //   { amount: NEAR.parse('18') }
  // );

  // // epoch unstake should not take effect now
  // await assertValidator(v1, '20', '0');
  // await assertValidator(v2, '40', '0');
  // await assertValidator(v3, '30', '30');

  // // fast-forward 
  // await owner.call(
  //   contract,
  //   'set_epoch_height',
  //   { epoch: 18 }
  // );

  // // only 12 NEAR left in stake now
  // await unstakeAll(owner, contract);
  // await assertValidator(v1, '20', '0');
  // await assertValidator(v2, '22', '18');
  // await assertValidator(v3, '30', '30');
});

skip('epoch collect rewards', async (test, {root, contract, alice, owner}) => {
  test.timeout(60 * 1000);
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
      weight: 20
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );
  await owner.call(
    contract,
    'add_validator',
    {
      validator_id: v3.accountId,
      weight: 30
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );

  // set manager
  const manager = await setManager(root, contract, owner);

  // update base stake amount of v1 to 10 NEAR
  await updateBaseStakeAmounts(
    contract,
    manager,
    [
      v1.accountId,
    ],
    [
      NEAR.parse("10")
    ]
  );

  // user stake
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse('60')
    }
  );

  // epoch stake
  await stakeAll(owner, contract);

  let total_share_amount_0 = NEAR.from(await contract.view('get_total_share_amount'));
  let total_near_amount_0 = NEAR.from(await contract.view('get_total_staked_balance'));
  test.truthy(total_share_amount_0.eq(NEAR.parse('70')));
  test.truthy(total_near_amount_0.eq(NEAR.parse('70')));

  // generate rewards
  await contract.call(
    v1,
    'add_reward',
    { amount: NEAR.parse('2').toString() }
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
  test.truthy(total_share_amount_1.eq(NEAR.parse('70')));
  test.truthy(total_near_amount_1.eq(NEAR.parse('77')));

  // set beneficiary
  await owner.call(
      contract,
      'set_beneficiary',
      {
          account_id: owner.accountId,
          bps: 1000
      }
  );

  // generate more rewards
  await contract.call(
    v1,
    'add_reward',
    { amount: NEAR.parse('2').toString() }
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
    '70177215189873417721518987'
  );
  test.is(
    total_near_amount_2.toString(),
    '79000000000000000000000000'
  );

  // check staked amount and base stake amount on each validator
  await assertValidator(v1, "24", "0", "12");
  await assertValidator(v2, "22", "0", "0");
  await assertValidator(v3, "33", "0", "0");
});

skip('epoch withdraw', async (test, {contract, alice, root, owner}) => {
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
      weight: 20
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );
  await owner.call(
    contract,
    'add_validator',
    {
      validator_id: v3.accountId,
      weight: 30
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
