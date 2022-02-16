import { Gas, NEAR, NearAccount, } from "near-workspaces-ava";
import { initWorkSpace, skip } from "./helper";

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
  contract: NearAccount
) {
  return async function (
    validator: NearAccount, 
    stakedAmount: string,
    unstakedAmount: string
  ) {
    test.is(
      await validator.view('get_account_staked_balance', { account_id: contract.accountId }),
      NEAR.parse(stakedAmount).toString()
    );
    test.is(
      await validator.view('get_account_unstaked_balance', { account_id: contract.accountId }),
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
  const assertValidator = assertValidatorAmountHelper(test, contract);

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
  const assertValidator = assertValidatorAmountHelper(test, contract);

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
  await assertValidator(v1, '5', '5');
  await assertValidator(v2, '10', '10');
  await assertValidator(v3, '15', '15');

  // unstake more
  await alice.call(
    contract,
    'unstake',
    { amount: NEAR.parse('18') }
  );

  // epoch unstake should not take effect now
  await unstakeAll(owner, contract);
  await assertValidator(v1, '5', '5');
  await assertValidator(v2, '10', '10');
  await assertValidator(v3, '15', '15');

  // fast-forward 
  await owner.call(
    contract,
    'set_epoch_height',
    { epoch: 18 }
  );

  // only 12 NEAR left in stake now
  await unstakeAll(owner, contract);
  await assertValidator(v1, '2', '8');
  await assertValidator(v2, '4', '16');
  await assertValidator(v3, '6', '24');
});
