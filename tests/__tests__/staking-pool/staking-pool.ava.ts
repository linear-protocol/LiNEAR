 import {Workspace, NEAR} from 'near-workspaces-ava';

const workspace = Workspace.init(async ({root}) => {
  const alice = await root.createAccount('alice');

  const contract = await root.createAndDeploy(
    'staking-pool',
    'compiled-contracts/staking_pool.wasm',
    {
      method: 'new',
      args: {},
    },
  );

  return {contract, alice};
});

workspace.test('check balance after initlization', async (test, {contract, alice}) => {
  // await root.call(contract, 'set_status', {message: 'lol'});
  test.is(
    await contract.view('get_account_staked_balance', {account_id: alice}),
    '0',
  );
  test.is(
    await contract.view('get_account_unstaked_balance', {account_id: alice}),
    '0',
  );
  test.is(
    await contract.view('get_account_total_balance', {account_id: alice}),
    '0',
  );
});

workspace.test('deposit and stake', async (test, {contract, alice}) => {
  const deposit = NEAR.parse('10');

  await alice.call(
    contract,
    'deposit',
    {},
    { attachedDeposit: deposit },
  );

  test.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    deposit.toString()
  );

  // stake
  const stakeAmount = NEAR.parse('9');
  await alice.call(
    contract,
    'stake',
    { amount: stakeAmount.toString() }
  );

  test.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    stakeAmount.toString()
  );
  test.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    deposit.sub(stakeAmount).toString()
  );
});
