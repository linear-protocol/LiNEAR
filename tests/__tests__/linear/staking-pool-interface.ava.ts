import { NEAR, Gas } from 'near-workspaces-ava';
import { initWorkSpace, assertFailure, epochHeightFastforward, epochStake } from './helper';

const ERR_UNSTAKED_BALANCE_NOT_AVAILABLE = 'The unstaked balance is not yet available due to unstaking delay';

const workspace = initWorkSpace();

workspace.test('check balances after initlization', async (test, {contract, alice}) => {
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
  // deposit
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
  let receivedLinearAmount = await alice.call(
    contract,
    'stake',
    { amount: stakeAmount.toString() }
  );
  test.is(
    stakeAmount.toString(),
    receivedLinearAmount.toString(),
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
  receivedLinearAmount = await alice.call(
    contract,
    'stake_all',
    {}
  );
  test.is(
    deposit.sub(stakeAmount).toString(),
    receivedLinearAmount.toString(),
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
  const receivedLinearAmount = await alice.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: stakeAmount },
  );
  test.is(
    stakeAmount.toString(),
    receivedLinearAmount.toString(),
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


workspace.test('unstake', async (test, { contract, alice }) => {
  // deposit
  const deposit = NEAR.parse('10');
  await alice.call(
    contract,
    'deposit',
    {},
    { attachedDeposit: deposit },
  );

  // stake
  const stakeAmount = NEAR.parse('9');
  await alice.call(
    contract,
    'stake',
    { amount: stakeAmount.toString() }
  );

  // unstake
  const unstakeAmount = NEAR.parse('5');
  await alice.call(
    contract,
    'unstake',
    { amount: unstakeAmount.toString() }
  );

  test.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    stakeAmount.sub(unstakeAmount).toString()
  );
  test.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    deposit.sub(stakeAmount).add(unstakeAmount).toString()
  );
});

workspace.test('unstake and withdraw', async (test, { contract, alice }) => {
  // deposit
  const deposit = NEAR.parse('10');
  await alice.call(
    contract,
    'deposit',
    {},
    { attachedDeposit: deposit },
  );

  // stake
  const stakeAmount = NEAR.parse('9');
  await alice.call(
    contract,
    'stake',
    { amount: stakeAmount.toString() }
  );

  // first withdraw
  const firstWithdrawAmount = NEAR.parse('0.5');
  await alice.call(
    contract,
    'withdraw',
    { amount: firstWithdrawAmount.toString() }
  );

  test.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    stakeAmount.toString()
  );
  test.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    deposit.sub(stakeAmount).sub(firstWithdrawAmount).toString()
  );

  // unstake
  const unstakeAmount = NEAR.parse('5');
  await alice.call(
    contract,
    'unstake',
    { amount: unstakeAmount.toString() }
  ); 

  // withdraw all immediately, should fail
  await assertFailure(
    test,
    alice.call(
      contract,
      'withdraw_all',
      {}
    ),
    ERR_UNSTAKED_BALANCE_NOT_AVAILABLE
  );

  // wait 4 epoches
  await epochHeightFastforward(contract, alice);

  // withdraw all after 4 epoches
  await alice.call(
    contract,
    'withdraw_all',
    {}
  );

  test.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    stakeAmount.sub(unstakeAmount).toString()
  );
  test.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    '0'
  );

  // unstake all
  await alice.call(
    contract,
    'unstake_all',
    {}
  );

  test.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    '0'
  );
  test.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    stakeAmount.sub(unstakeAmount).toString()
  );

  // wait 4 epoches
  await epochHeightFastforward(contract, alice);

  // withdraw all after 4 epoches
  const withdrawAmount = NEAR.parse('1');
  await alice.call(
    contract,
    'withdraw',
    { amount: withdrawAmount.toString() }
  );

  test.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    '0'
  );
  test.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    stakeAmount.sub(unstakeAmount).sub(withdrawAmount).toString()
  );

});

workspace.test('late unstake and withdraw', async (test, { contract ,alice }) => {
  // deposit
  const deposit = NEAR.parse('10');
  await alice.call(
    contract,
    'deposit',
    {},
    { attachedDeposit: deposit },
  );

  // stake
  const stakeAmount = NEAR.parse('9');
  await alice.call(
    contract,
    'stake',
    { amount: stakeAmount.toString() }
  );

  // call epoch_stake, in order to trigger clean up
  await epochStake(alice, contract);

  // unstake
  const unstakeAmount = NEAR.parse('5');
  await alice.call(
    contract,
    'unstake',
    { amount: unstakeAmount.toString() }
  ); 

  // withdraw available time should be 5 epoches later
  const account: any = await contract.view(
    'get_account_details',
    {
      account_id: alice.accountId
    }
  );

  test.is(
    account.unstaked_available_epoch_height,
    15
  );

  // cannot withdraw after 4 epoches
  await epochHeightFastforward(contract, alice);

  await assertFailure(
    test,
    alice.call(
      contract,
      'withdraw',
      { amount: unstakeAmount.toString() }
    ),
    'The unstaked balance is not yet available due to unstaking delay'
  );

  // wait for one more epoch
  await epochHeightFastforward(contract, alice, 1);

  await alice.call(
    contract,
    'withdraw',
    { amount: unstakeAmount.toString() }
  );
});
