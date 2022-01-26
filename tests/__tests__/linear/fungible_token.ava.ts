 import {Workspace, NEAR} from 'near-workspaces-ava';

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
        reward_fee_fraction: {
          numerator: 1,
          denominator: 100 
        }
      },
    },
  );

  return {contract, alice};
});

workspace.test('get ft_metadata after initialization', async (test, {contract, alice}) => {
  const metadata = await contract.view('ft_metadata', {}) as any;
  test.is(
    metadata.symbol,
    'LINEAR',
  );
  // test.is(
  //   await contract.view('get_account_unstaked_balance', {account_id: alice}),
  //   '0',
  // );
  // test.is(
  //   await contract.view('get_account_total_balance', {account_id: alice}),
  //   '0',
  // );
});

// workspace.test('deposit and stake', async (test, {contract, alice}) => {
//   const deposit = NEAR.parse('10');

//   await alice.call(
//     contract,
//     'deposit',
//     {},
//     { attachedDeposit: deposit },
//   );

//   test.is(
//     await contract.view('get_account_unstaked_balance', { account_id: alice }),
//     deposit.toString()
//   );

//   // stake
//   const stakeAmount = NEAR.parse('9');
//   await alice.call(
//     contract,
//     'stake',
//     { amount: stakeAmount.toString() }
//   );

//   test.is(
//     await contract.view('get_account_staked_balance', { account_id: alice }),
//     stakeAmount.toString()
//   );
//   test.is(
//     await contract.view('get_account_unstaked_balance', { account_id: alice }),
//     deposit.sub(stakeAmount).toString()
//   );
// });

// workspace.test('add reward', async (test, { contract, alice }) => {
//   // deposit
//   const deposit = NEAR.parse('10');

//   await alice.call(
//     contract,
//     'deposit',
//     {},
//     { attachedDeposit: deposit },
//   );

//   // stake
//   const stakeAmount = NEAR.parse('9');
//   await alice.call(
//     contract,
//     'stake',
//     { amount: stakeAmount.toString() }
//   );

//   // add reward
//   const reward = NEAR.parse('1');
//   await alice.call(
//     contract,
//     'add_reward',
//     { amount: reward.toString() }
//   );

//   test.is(
//     await contract.view('get_account_staked_balance', { account_id: alice }),
//     stakeAmount.add(reward).toString()
//   );
//   test.is(
//     await contract.view('get_account_unstaked_balance', { account_id: alice }),
//     deposit.sub(stakeAmount).toString()
//   );
// });

// workspace.test('unstake', async (test, { contract, alice }) => {
//   // deposit
//   const deposit = NEAR.parse('10');

//   await alice.call(
//     contract,
//     'deposit',
//     {},
//     { attachedDeposit: deposit },
//   );

//   // stake
//   const stakeAmount = NEAR.parse('9');
//   await alice.call(
//     contract,
//     'stake',
//     { amount: stakeAmount.toString() }
//   );

//   // add reward
//   const reward = NEAR.parse('1');
//   await alice.call(
//     contract,
//     'add_reward',
//     { amount: reward.toString() }
//   );

//   // unstake
//   const unstakeAmount = NEAR.parse('5');
//   await alice.call(
//     contract,
//     'unstake',
//     { amount: unstakeAmount.toString() }
//   );

//   test.is(
//     await contract.view('get_account_staked_balance', { account_id: alice }),
//     stakeAmount.add(reward).sub(unstakeAmount).toString()
//   );
//   test.is(
//     await contract.view('get_account_unstaked_balance', { account_id: alice }),
//     deposit.sub(stakeAmount).add(unstakeAmount).toString()
//   );
// });

// workspace.test('withdraw', async (test, { contract, alice }) => {
//   // deposit
//   const deposit = NEAR.parse('10');

//   await alice.call(
//     contract,
//     'deposit',
//     {},
//     { attachedDeposit: deposit },
//   );

//   // stake
//   const stakeAmount = NEAR.parse('9');
//   await alice.call(
//     contract,
//     'stake',
//     { amount: stakeAmount.toString() }
//   );

//   // add reward
//   const reward = NEAR.parse('1');
//   await alice.call(
//     contract,
//     'add_reward',
//     { amount: reward.toString() }
//   );

//   // first withdraw
//   const firstWithdrawAmount = NEAR.parse('0.5');
//   await alice.call(
//     contract,
//     'withdraw',
//     { amount: firstWithdrawAmount.toString() }
//   );

//   test.is(
//     await contract.view('get_account_staked_balance', { account_id: alice }),
//     stakeAmount.add(reward).toString()
//   );
//   test.is(
//     await contract.view('get_account_unstaked_balance', { account_id: alice }),
//     deposit.sub(stakeAmount).sub(firstWithdrawAmount).toString()
//   );

//   // unstake
//   const unstakeAmount = NEAR.parse('5');
//   await alice.call(
//     contract,
//     'unstake',
//     { amount: unstakeAmount.toString() }
//   ); 

//   // withdraw all
//   await alice.call(
//     contract,
//     'withdraw_all',
//     {}
//   );

//   test.is(
//     await contract.view('get_account_staked_balance', { account_id: alice }),
//     stakeAmount.add(reward).sub(unstakeAmount).toString()
//   );
//   test.is(
//     await contract.view('get_account_unstaked_balance', { account_id: alice }),
//     '0'
//   );
// });
