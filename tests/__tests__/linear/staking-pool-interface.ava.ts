import { NEAR, Gas } from 'near-workspaces';
import {
  initWorkspace,
  assertFailure,
  epochHeightFastforward,
  epochStake,
  test,
} from './helper';

const ERR_NON_POSITIVE_DEPOSIT_AMOUNT = "Deposit amount should be positive";
const ERR_UNSTAKED_BALANCE_NOT_AVAILABLE =
  'The unstaked balance is not yet available due to unstaking delay';

test.beforeEach(async (t) => {
  t.context = await initWorkspace();
});

test.afterEach(async (t) => {
  await t.context.worker.tearDown();
});

test('check balances after initialization', async (t) => {
  const { contract, alice } = t.context;
  t.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    '0',
  );
  t.is(
    await contract.view('get_account_unstaked_balance', {
      account_id: alice,
    }),
    '0',
  );
  t.is(
    await contract.view('get_account_total_balance', { account_id: alice }),
    '0',
  );
});

test.only('deposit 0 NEAR is not allowed', async (t) => {
  const { contract, alice } = t.context;
  // deposit 0 NEAR will fail
  await assertFailure(
    t,
    alice.call(contract, 'deposit', {}, { attachedDeposit: '0' }),
    ERR_NON_POSITIVE_DEPOSIT_AMOUNT
  );
});

test('deposit first and stake later', async (t) => {
  const { contract, alice } = t.context;
  // deposit
  const deposit = NEAR.parse('10');
  await alice.call(contract, 'deposit', {}, { attachedDeposit: deposit });

  t.is(
    await contract.view('get_account_unstaked_balance', {
      account_id: alice,
    }),
    deposit.toString(),
  );

  // stake
  const stakeAmount = NEAR.parse('9');
  let receivedLinearAmount = await alice.call<string>(contract, 'stake', {
    amount: stakeAmount.toString(),
  });
  t.is(stakeAmount.toString(), receivedLinearAmount.toString());

  t.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    stakeAmount.toString(),
  );
  t.is(
    await contract.view('get_account_unstaked_balance', {
      account_id: alice,
    }),
    deposit.sub(stakeAmount).toString(),
  );

  // stake all
  receivedLinearAmount = await alice.call(contract, 'stake_all', {});
  t.is(deposit.sub(stakeAmount).toString(), receivedLinearAmount.toString());

  t.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    deposit.toString(),
  );
  t.is(
    await contract.view('get_account_unstaked_balance', {
      account_id: alice,
    }),
    deposit.sub(deposit).toString(),
  );
});

test('deposit and stake', async (t) => {
  const { contract, alice } = t.context;

  // deposit and stake
  const stakeAmount = NEAR.parse('10');
  const receivedLinearAmount = await alice.call<string>(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: stakeAmount },
  );
  t.is(stakeAmount.toString(), receivedLinearAmount.toString());

  t.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    stakeAmount.toString(),
  );
  t.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    stakeAmount.sub(stakeAmount).toString(),
  );
});

test('unstake', async (t) => {
  const { contract, alice } = t.context;
  // deposit
  const deposit = NEAR.parse('10');
  await alice.call(contract, 'deposit', {}, { attachedDeposit: deposit });

  // stake
  const stakeAmount = NEAR.parse('9');
  await alice.call(contract, 'stake', { amount: stakeAmount.toString() });

  // unstake
  const unstakeAmount = NEAR.parse('5');
  await alice.call(contract, 'unstake', { amount: unstakeAmount.toString() });

  t.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    stakeAmount.sub(unstakeAmount).toString(),
  );
  t.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    deposit.sub(stakeAmount).add(unstakeAmount).toString(),
  );
});

test('unstake and withdraw', async (t) => {
  const { contract, alice } = t.context;
  // deposit
  const deposit = NEAR.parse('10');
  await alice.call(contract, 'deposit', {}, { attachedDeposit: deposit });

  // stake
  const stakeAmount = NEAR.parse('9');
  await alice.call(contract, 'stake', { amount: stakeAmount.toString() });

  // first withdraw
  const firstWithdrawAmount = NEAR.parse('0.5');
  await alice.call(contract, 'withdraw', {
    amount: firstWithdrawAmount.toString(),
  });

  t.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    stakeAmount.toString(),
  );
  t.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    deposit.sub(stakeAmount).sub(firstWithdrawAmount).toString(),
  );

  // unstake
  const unstakeAmount = NEAR.parse('5');
  await alice.call(contract, 'unstake', { amount: unstakeAmount.toString() });

  // withdraw all immediately, should fail
  await assertFailure(
    t,
    alice.call(contract, 'withdraw_all', {}),
    ERR_UNSTAKED_BALANCE_NOT_AVAILABLE,
  );

  // wait 4 epoches
  await epochHeightFastforward(contract, alice);

  // withdraw all after 4 epoches
  await alice.call(contract, 'withdraw_all', {});

  t.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    stakeAmount.sub(unstakeAmount).toString(),
  );
  t.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    '0',
  );

  // unstake all
  await alice.call(contract, 'unstake_all', {});

  t.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    '0',
  );
  t.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    stakeAmount.sub(unstakeAmount).toString(),
  );

  // wait 4 epoches
  await epochHeightFastforward(contract, alice);

  // withdraw all after 4 epoches
  const withdrawAmount = NEAR.parse('1');
  await alice.call(contract, 'withdraw', { amount: withdrawAmount.toString() });

  t.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    '0',
  );
  t.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    stakeAmount.sub(unstakeAmount).sub(withdrawAmount).toString(),
  );
});

test('late unstake and withdraw', async (t) => {
  const { contract, alice } = t.context;
  // deposit
  const deposit = NEAR.parse('10');
  await alice.call(contract, 'deposit', {}, { attachedDeposit: deposit });

  // stake
  const stakeAmount = NEAR.parse('9');
  await alice.call(contract, 'stake', { amount: stakeAmount.toString() });

  // call epoch_stake, in order to trigger clean up
  await epochStake(alice, contract);

  // unstake
  const unstakeAmount = NEAR.parse('5');
  await alice.call(contract, 'unstake', { amount: unstakeAmount.toString() });

  // withdraw available time should be 5 epoches later
  const account: any = await contract.view('get_account_details', {
    account_id: alice.accountId,
  });

  t.is(account.unstaked_available_epoch_height, 15);

  // cannot withdraw after 4 epoches
  await epochHeightFastforward(contract, alice);

  await assertFailure(
    t,
    alice.call(contract, 'withdraw', { amount: unstakeAmount.toString() }),
    'The unstaked balance is not yet available due to unstaking delay',
  );

  // wait for one more epoch
  await epochHeightFastforward(contract, alice, 1);

  await alice.call(contract, 'withdraw', {
    amount: unstakeAmount.toString(),
  });
});
