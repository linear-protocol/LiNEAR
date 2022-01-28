import { Workspace, NEAR, NearAccount } from 'near-workspaces-ava';

const workspace = Workspace.init(async ({root}) => {
  const owner = await root.createAccount('linear_owner');
  const alice = await root.createAccount('alice');

  const contract = await root.createAndDeploy(
    'linear',
    'compiled-contracts/linear.wasm',
    {
      method: 'new',
      args: {
        owner_id: 'linear_owner',
        reward_fee: {
          numerator: 1,
          denominator: 100 
        }
      },
    },
  );

  return { contract, alice };
});

workspace.test('contract initlization', async (test, {contract, alice}) => {
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

workspace.test('deposit first and stake later', async (test, {contract, alice}) => {
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

  // stake all
  await alice.call(
    contract,
    'stake_all',
    {}
  );

  test.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    deposit.toString()
  );
  test.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    deposit.sub(deposit).toString()
  );
});

workspace.test('deposit and stake', async (test, {contract, alice}) => {
  // deposit and stake
  const stakeAmount = NEAR.parse('10');
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: stakeAmount },
  );

  test.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    stakeAmount.toString()
  );
  test.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    stakeAmount.sub(stakeAmount).toString()
  );
});