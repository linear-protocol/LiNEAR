import {NEAR, NearAccount, Worker} from 'near-workspaces';
import {assertFailure, createAndDeploy} from '../linear/helper';

import anyTest, { TestFn } from "ava";

const test = anyTest as TestFn<WorkSpace>;

interface WorkSpace {
  worker: Worker,
  contract: NearAccount,
  alice: NearAccount,
}

async function initWorkSpace(): Promise<WorkSpace> {
  const worker = await Worker.init({
    network: 'sandbox',
    rm: true,
  });

  const root = worker.rootAccount;

  const alice = await root.createSubAccount('alice');

  const contract = await createAndDeploy(
    root,
    'staking-pool',
    'compiled-contracts/mock_staking_pool.wasm',
    {
      methodName: 'new',
      args: {},
    },
  )

  return { worker, contract, alice };
}

test(
  'check balance after initlization',
  async (t) => {
    const { contract, alice } = t.context;
    // await root.call(contract, 'set_status', {message: 'lol'});
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
  },
);

test.before(async (t) => {
  t.context = await initWorkSpace();
});

test.after(async (t) => {
  await t.context.worker.tearDown();
});

test('deposit and stake', async (t) => {
  const { contract, alice } = t.context;
  const deposit = NEAR.parse('10');

  await alice.call(contract, 'deposit', {}, { attachedDeposit: deposit });

  t.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    deposit.toString(),
  );

  // stake
  const stakeAmount = NEAR.parse('9');
  await alice.call(contract, 'stake', { amount: stakeAmount.toString() });

  t.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    stakeAmount.toString(),
  );
  t.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    deposit.sub(stakeAmount).toString(),
  );
});

test('add reward', async (t) => {
  const { contract, alice } = t.context;
  // deposit
  const deposit = NEAR.parse('10');

  await alice.call(contract, 'deposit', {}, { attachedDeposit: deposit });

  // stake
  const stakeAmount = NEAR.parse('9');
  await alice.call(contract, 'stake', { amount: stakeAmount.toString() });

  // add reward
  const reward = NEAR.parse('1');
  await alice.call(contract, 'add_reward', { amount: reward.toString() });

  t.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    stakeAmount.add(reward).toString(),
  );
  t.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    deposit.sub(stakeAmount).toString(),
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

  // add reward
  const reward = NEAR.parse('1');
  await alice.call(contract, 'add_reward', { amount: reward.toString() });

  // unstake
  const unstakeAmount = NEAR.parse('5');
  await alice.call(contract, 'unstake', { amount: unstakeAmount.toString() });

  t.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    stakeAmount.add(reward).sub(unstakeAmount).toString(),
  );
  t.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    deposit.sub(stakeAmount).add(unstakeAmount).toString(),
  );
});

test('withdraw', async (t) => {
  const { contract, alice } = t.context;
  // deposit
  const deposit = NEAR.parse('10');

  await alice.call(contract, 'deposit', {}, { attachedDeposit: deposit });

  // stake
  const stakeAmount = NEAR.parse('9');
  await alice.call(contract, 'stake', { amount: stakeAmount.toString() });

  // add reward
  const reward = NEAR.parse('1');
  await alice.call(contract, 'add_reward', { amount: reward.toString() });

  // first withdraw
  const firstWithdrawAmount = NEAR.parse('0.5');
  await alice.call(contract, 'withdraw', {
    amount: firstWithdrawAmount.toString(),
  });

  t.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    stakeAmount.add(reward).toString(),
  );
  t.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    deposit.sub(stakeAmount).sub(firstWithdrawAmount).toString(),
  );

  // unstake
  const unstakeAmount = NEAR.parse('5');
  await alice.call(contract, 'unstake', { amount: unstakeAmount.toString() });

  // withdraw all
  await alice.call(contract, 'withdraw_all', {});

  t.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    stakeAmount.add(reward).sub(unstakeAmount).toString(),
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
    stakeAmount.add(reward).sub(unstakeAmount).toString(),
  );

  // withdraw all
  const withdrawAmount = NEAR.parse('1');
  await alice.call(contract, 'withdraw', { amount: withdrawAmount.toString() });

  t.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    '0',
  );
  t.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    stakeAmount.add(reward).sub(unstakeAmount).sub(withdrawAmount).toString(),
  );
});

test('panic', async (t) => {
  const { contract, alice } = t.context;
  await alice.call(contract, 'set_panic', { panic: true });

  await assertFailure(
    t,
    alice.call(contract, 'deposit', {}, { attachedDeposit: NEAR.parse('10') }),
    'Test Panic!',
  );

  await assertFailure(
    t,
    alice.call(contract, 'stake', { amount: NEAR.parse('4') }),
    'Test Panic!',
  );

  await assertFailure(
    t,
    alice.call(contract, 'withdraw', { amount: NEAR.parse('1') }),
    'Test Panic!',
  );

  await assertFailure(
    t,
    alice.call(contract, 'unstake', { amount: NEAR.parse('1') }),
    'Test Panic!',
  );
});
