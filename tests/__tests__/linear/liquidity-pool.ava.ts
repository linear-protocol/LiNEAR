import { NEAR, BN } from 'near-workspaces-ava';
import { initWorkSpace, assertFailure } from './helper';

// Liquidity pool swap fee constants
const MAX_FEE = new BN(300); // 10,000
const MIN_FEE = new BN(30);  // 10,000
const TARGET_NEAR_AMOUNT = NEAR.parse('10000');
// Estimate swap fee
const estimateSwapFee = (totalAmount: NEAR, amount: NEAR) => {
  let diff = MAX_FEE.sub(MIN_FEE);
  return amount.mul(
    MAX_FEE.sub(
      diff.mul(totalAmount.sub(amount)).div(TARGET_NEAR_AMOUNT)
    )
  ).div(new BN(10000));
}

const workspace = initWorkSpace();

workspace.test('add initial liquidity', async (test, { contract, alice, bob }) => {
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

workspace.test('instant unstake', async (test, { contract, alice, bob }) => {
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

  const totalAmount = addedLiqudityAmount;

  // Alice requests instant unstake
  const delta = NEAR.parse('0.5');
  const unstakeAmount = NEAR.parse('5');
  let fee = estimateSwapFee(totalAmount, unstakeAmount);
  const receivedAmount: string = await alice.call(
    contract,
    'instant_unstake',
    {
      staked_shares_in: unstakeAmount.toString(),
      min_amount_out: unstakeAmount.sub(delta).toString()
    }
  );
  test.is(
    unstakeAmount.sub(fee).toString(),
    NEAR.from(receivedAmount).toString()
  );

  // Alice requests another instant unstake
  const unstakeAmount2 = NEAR.parse('3');
  fee = estimateSwapFee(totalAmount, unstakeAmount2);
  const receivedAmount2: string = await alice.call(
    contract,
    'instant_unstake',
    {
      staked_shares_in: unstakeAmount2.toString(),
      min_amount_out: unstakeAmount2.sub(delta).toString()
    }
  );
  test.is(
    unstakeAmount2.sub(fee).toString(),
    NEAR.from(receivedAmount2).toString()
  );

});

// workspace.test('remove liquidity', async (test, { contract, alice }) => {


// });