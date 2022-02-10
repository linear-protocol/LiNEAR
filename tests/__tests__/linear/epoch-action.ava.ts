import { Gas, NEAR, NearAccount, } from "near-workspaces-ava";
import { initWorkSpace } from "./helper";

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
