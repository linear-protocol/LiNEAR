import { BN, Gas, NEAR, NearAccount, stake, } from "near-workspaces-ava";
import {
  assertFailure,
  initWorkSpace,
  createStakingPool,
  updateBaseStakeAmounts,
  setManager,
  assertValidatorAmountHelper,
  skip,
  epochStake,
  epochUnstake
} from "./helper";

const workspace = initWorkSpace();

async function stakeAll (owner: NearAccount, contract: NearAccount) {
  let run = true;
  while (run) {
    run = await epochStake(owner, contract);
  }
}

async function unstakeAll (owner: NearAccount, contract: NearAccount) {
  let run = true;
  while (run) {
    run = await epochUnstake(owner, contract);
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
  await alice.call(
    contract,
    'unstake',
    { amount: NEAR.parse('18') }
  );

  // epoch unstake should not take effect now
  await assertValidator(v1, '20', '0');
  await assertValidator(v2, '32.5', '7.5');
  await assertValidator(v3, '37.5', '22.5');

  // fast-forward 
  await owner.call(
    contract,
    'set_epoch_height',
    { epoch: 18 }
  );

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
    [
      v1.accountId,
    ],
    [
      NEAR.parse("10")
    ]
  );

  // fast-forward
  await owner.call(
    contract,
    'set_epoch_height',
    { epoch: 22 }
  );

  // unstake more; remaining total staked: 120 - 30 - 18 - 26 = 46
  await alice.call(
    contract,
    'unstake',
    { amount: NEAR.parse('26') }
  );

  // epoch unstake
  await unstakeAll(owner, contract);

  // validators should have target stake amount based on weights + base stake amounts
  // - 1st epoch_unstake() unstaked 19.5 NEAR (amount = delta) from validator v3
  // - 2nd epoch_unstake() unstaked 6.5 NEAR (amount = rest) from validator v2
  await assertValidator(v1, '12', '8', '10', '16');   // target = 10 (base) + 6 (weighted) = 16; delta (1st) = 12 - 16 = -4; delta (2nd) = 12 - 16 = -4;
  await assertValidator(v2, '16', '24', '0', '12');  // target = 12 (weighted); delta (1st) = 22.5 - 12 = 10.5; delta (2nd) = 16 - 12 = 4; 
  await assertValidator(v3, '18', '42', '0', '18');   // target = 18 (weighted); delta (1st) = 37.5 - 18 = 19.5; delta (2nd) = 18 - 18 = 0;

 
  // reset base stake amount of v1 to 0
  await updateBaseStakeAmounts(
    contract,
    manager,
    [
      v1.accountId,
    ],
    [
      NEAR.parse("0")
    ]
  );

  // fast-forward
  await owner.call(
    contract,
    'set_epoch_height',
    { epoch: 26 }
  );

  // unstake more; remaining total staked: 120 - 30 - 18 - 26 - 10 = 36
  await alice.call(
    contract,
    'unstake',
    { amount: NEAR.parse('10') }
  );

  // epoch unstake
  await unstakeAll(owner, contract);

  // validators should have target stake amount based on weights + base stake amounts
  // - 1st epoch_unstake() unstaked 6 NEAR (amount = delta) from validator v1;
  // - 2nd epoch_unstake() unstaked 4 NEAR (amount = rest) from validator v2;
  await assertValidator(v1, '6', '14', '0', '6');   // target = 6 (weighted); delta (1st) = 12 - 6 = 6; delta (2nd) = 6 - 6 = 0;
  await assertValidator(v2, '12', '28', '0', '12');  // target = 12 (weighted); delta (1st) = 16 - 12 = 4; delta (2nd) = 12 - 12 = 0;
  await assertValidator(v3, '18', '42', '0', '18');   // target = 18 (weighted); delta (1st) = 18 - 18 = 0; delta (2nd) = 18 - 18 = 0; 
});

workspace.test('epoch collect rewards', async (test, {root, contract, alice, owner}) => {
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

  await assertValidator(v1, '20', '0');
  await assertValidator(v2, '32.5', '7.5');
  await assertValidator(v3, '37.5', '22.5');

  // withdraw should fail now
  await assertFailure(
    test,
    owner.call(
      contract,
      'epoch_withdraw',
      {
        validator_id: v2.accountId
      },
      {
        gas: Gas.parse('200 Tgas')
      }
    ),
    'Cannot withdraw from a pending release validator'
  );

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
      validator_id: v2.accountId
    },
    {
      gas: Gas.parse('200 Tgas')
    }
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

  await assertValidator(v1, '20', '0');
  await assertValidator(v2, '32.5', '0');
  await assertValidator(v3, '37.5', '0');
});

skip('estimate gas of epoch unstake', async (test, {contract, alice, root, owner}) => {
  const validatorsNum = 255

  const names = Array.from({ length: validatorsNum }, (_, index) => `v${index + 1}`)
  const weights = names.map((_, index) => Math.floor(index * 51 / validatorsNum))

  const shuffleArray = (array: any[]) => {
    for (let i = array.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [array[i], array[j]] = [array[j], array[i]];
    }
  }

  shuffleArray(weights)

  const validators: NearAccount[] = []
  for (const name of names) {
    validators.push(await createStakingPool(root, name))
  }

  let sliceIndex = 0
  const sliceSize = 6
  while(sliceIndex < validators.length) {
    const validatorIdSlice = validators.slice(sliceIndex, sliceSize).map(v => v.accountId)
    const weightSlice = weights.slice(sliceIndex, sliceSize)
    await owner.call(
      contract,
      'add_validators',
      {
        validator_ids: validatorIdSlice,
        weights: weightSlice
      },
      {
        gas: Gas.parse('300 Tgas')
      }
    )
    sliceIndex += sliceSize
  }

  // user stake
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse('10000')
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
    { amount: NEAR.parse('2000') }
  );

  // epoch unstake
  let run = true;
  const gasBurnts: Gas[] = []
  while (run) {
    const outcome = await owner.call_raw(
      contract,
      'epoch_unstake',
      {},
      {
        gas: Gas.parse('280 Tgas')
      }
    );

    const receiptsGasBurnt = outcome.result.receipts_outcome
      .map(receipt => receipt.outcome.gas_burnt)
      .reduce((pre, gas) => pre + gas)
    gasBurnts.push(outcome.gas_burnt.add(new BN(receiptsGasBurnt)))
    const json = Buffer.from(outcome.SuccessValue, 'base64').toString()
    run = JSON.parse(json)
  }
  console.log(gasBurnts.map(gas => gas.toBigInt()));
});
