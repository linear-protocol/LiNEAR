import { NEAR } from 'near-workspaces-ava';
import { initWorkSpace, assertFailure } from './helper';

const workspace = initWorkSpace();

workspace.test('add liquidity', async (test, { contract, alice, bob }) => {
  // Alice deposits and stakes to avoid empty stake shares
  const stakeAmount = NEAR.parse('10');
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: stakeAmount },
  );
  
  // Bob adds liquidity
  const addedLiqudityAmount = NEAR.parse('50');
  await bob.call(
    contract,
    'add_liquidity',
    {},
    { attachedDeposit: addedLiqudityAmount },
  );

  test.is(
    (await contract.view('get_account', { account_id: bob }) as any).liquidity_pool_share,
    addedLiqudityAmount.toString()
  );

  // Alice also adds liquidity
  const addedLiqudityAmount2 = NEAR.parse('20');
  await alice.call(
    contract,
    'add_liquidity',
    {},
    { attachedDeposit: addedLiqudityAmount2 },
  );

  test.is(
    (await contract.view('get_account', { account_id: alice }) as any).liquidity_pool_share,
    addedLiqudityAmount2.toString()
  );

});

// workspace.test('instant unstake', async (test, { contract, alice, bob }) => {
  // // deposit and stake
  // const stakeAmount = NEAR.parse('10');
  // await alice.call(
  //   contract,
  //   'deposit_and_stake',
  //   {},
  //   { attachedDeposit: stakeAmount },
  // );

  // // add liquidity
  // const addedLiqudityAmount = NEAR.parse('50');
  // await bob.call(
  //   contract,
  //   'add_liquidity',
  //   {},
  //   { attachedDeposit: addedLiqudityAmount },
  // );

  // // deposit and stake
  // await alice.call(
  //   contract,
  //   'deposit_and_stake',
  //   {},
  //   { attachedDeposit: stakeAmount },
  // );

  // // instant unstake
  // const unstakeAmount = NEAR.parse('5');
  // const receivedAmount = await alice.call(
  //   contract,
  //   'instant_unstake',
  //   { amount: unstakeAmount.toString() }
  // );

  // console.log('receivedAmount', receivedAmount);


// });

// workspace.test('remove liquidity', async (test, { contract, alice }) => {


// });