import { NEAR, BN } from 'near-workspaces-ava';
import {
  initWorkSpace,
  callWithMetrics,
  numbersEqual,
  noMoreThanOneYoctoDiff,
  assertFailure,
  ONE_YOCTO
} from './helper';

// helper functions

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

const getBalance = async (user) => {
  const balance = await user.balance();
  return balance.total
}

const getSummary = async (contract) => {
  return await contract.view("get_summary", {}) as any;
}

const getTotalStakedNEAR = async (contract) => {
  return NEAR.from((await getSummary(contract)).total_staked_near_amount);
}

const getPoolValue = async (contract) => {
  const summary = await getSummary(contract);
  const { lp_near_amount, ft_price, lp_staked_share } = summary;
  const price = NEAR.from(ft_price).div(NEAR.parse('1'));
  return NEAR.from(lp_near_amount).add(
    NEAR.from(lp_staked_share).mul(price)
  );
}

const ftPrice = async(contract) => {
  return NEAR.from(await contract.view('ft_price', {})).div(NEAR.parse('1'));
}

const stake = async(test, {contract, user, amount}) => {
  await user.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: amount },
  );
}

const addLiquidity = async(test, {contract, user, amount}) => {
  const previousPoolValue = await getPoolValue(contract);
  await user.call(
    contract,
    'add_liquidity',
    {},
    { attachedDeposit: amount },
  );
  const updatedPoolValue = await getPoolValue(contract);
  test.is(
    (await contract.view('get_account', { account_id: user }) as any).liquidity_pool_share_value,
    amount.toString()
  );
  test.is(
    previousPoolValue.add(amount).toString(),
    updatedPoolValue.toString()
  );
}

const removeLiqudity = async (test, {contract, user, amount}) => {
  const previousPoolValue = await getPoolValue(contract);
  const balance = await getBalance(user);
  const result = await callWithMetrics(
    user,
    contract,
    'remove_liquidity',
    { amount },
    { attachedDeposit: ONE_YOCTO }
  );
  const updatedPoolValue = await getPoolValue(contract);

  const receivedAmount = NEAR.from(result.successValue[0]);
  const receivedStakedShare = NEAR.from(result.successValue[1]);
  const price = await ftPrice(contract);
  noMoreThanOneYoctoDiff(
    test,
    amount,
    receivedAmount.add(receivedStakedShare.mul(price))
  );
  noMoreThanOneYoctoDiff(
    test,
    previousPoolValue.sub(amount),
    updatedPoolValue
  );
  // Fuzzy match due to balance accuracy issue
  numbersEqual(
    test,
    balance.add(receivedAmount).sub(result.metrics.tokensBurnt),
    await getBalance(user),
    0.02
  );
}

const instantUnstake = async (test, {contract, user, amount}) => {
  const delta = NEAR.parse('0.5');
  const summary = await getSummary(contract);
  const totalAmount = NEAR.from((summary as any).lp_near_amount);
  let fee = estimateSwapFee(totalAmount, amount);
  const receivedAmount: string = await user.call(
    contract,
    'instant_unstake',
    {
      stake_shares_in: amount.toString(),
      min_amount_out: amount.sub(delta).toString()
    }
  );
  test.is(
    amount.sub(fee).toString(),
    NEAR.from(receivedAmount).toString()
  );
}

// test cases

const workspace = initWorkSpace();

workspace.test('add initial liquidity', async (test, { contract, alice, bob }) => {
  // Alice deposits and stakes to avoid empty stake shares
  await stake(test, {
    contract,
    user: alice,
    amount: NEAR.parse('10')
  });

  // Bob adds initial liquidity
  await addLiquidity(test, {
    contract,
    user: bob,
    amount: NEAR.parse('50')
  });

  // Alice also adds liquidity
  await addLiquidity(test, {
    contract,
    user: alice,
    amount: NEAR.parse('20')
  });
});

workspace.test('instant unstake', async (test, { contract, alice, bob }) => {
  // Alice deposits and stakes to avoid empty stake shares
  await stake(test, {
    contract,
    user: alice,
    amount: NEAR.parse('10')
  });

  // Bob adds initial liquidity
  await addLiquidity(test, {
    contract,
    user: bob,
    amount: NEAR.parse('50')
  });

  // Alice requests instant unstake
  await instantUnstake(test, {
    contract,
    user: alice,
    amount: NEAR.parse('5')
  });

  // Alice requests another instant unstake
  await instantUnstake(test, {
    contract,
    user: alice,
    amount: NEAR.parse('3')
  });
});

workspace.test('remove liquidity', async (test, { contract, alice, bob }) => {
  // Alice deposits and stakes to avoid empty stake shares
  await stake(test, {
    contract,
    user: alice,
    amount: NEAR.parse('10')
  });

  // Bob adds initial liquidity
  await addLiquidity(test, {
    contract,
    user: bob,
    amount: NEAR.parse('50')
  });

  // Bob removes liquidity from pool for the 1st time
  await removeLiqudity(test, {
    contract,
    user: bob,
    amount: NEAR.parse('10')
  });

  // Bob removes liquidity from pool for the 2nd time
  await removeLiqudity(test, {
    contract,
    user: bob,
    amount: NEAR.parse('5')
  });
});

workspace.test('issue: add liquidity precision loss', async (test, { contract, alice, bob }) => {
  // Alice deposits and stakes to avoid empty stake shares
  await stake(test, {
    contract,
    user: alice,
    amount: NEAR.parse('10')
  });

  // Bob adds initial liquidity
  await addLiquidity(test, {
    contract,
    user: bob,
    amount: NEAR.parse('50')
  });

  // Alice requests instant unstake
  await instantUnstake(test, {
    contract,
    user: alice,
    amount: NEAR.parse('5')
  });

  // Alice also adds liquidity, here may introduce precision loss
  await addLiquidity(test, {
    contract,
    user: alice,
    amount: NEAR.parse('20')
  });
});

workspace.test('issue: remove liquidity precision loss', async (test, { contract, alice, bob }) => {
  // Alice deposits and stakes to avoid empty stake shares
  await stake(test, {
    contract,
    user: alice,
    amount: NEAR.parse('10')
  });

  // Bob adds initial liquidity
  await addLiquidity(test, {
    contract,
    user: bob,
    amount: NEAR.parse('50')
  });

  // Alice requests instant unstake
  await instantUnstake(test, {
    contract,
    user: alice,
    amount: NEAR.parse('5')
  });

  // Bob removes liquidity, here may introduce precision loss
  await removeLiqudity(test, {
    contract,
    user: bob,
    amount: NEAR.parse('20')
  });

  // Alice requests instant unstake
  await instantUnstake(test, {
    contract,
    user: alice,
    amount: NEAR.parse('5')
  });

  // Bob removes liquidity, here may introduce precision loss
  await removeLiqudity(test, {
    contract,
    user: bob,
    amount: NEAR.parse('15')
  });
});

workspace.test('rebalance liquidity', async (test, { contract, alice, bob }) => {
  // Alice deposits and stakes to avoid empty stake shares
  await stake(test, {
    contract,
    user: alice,
    amount: NEAR.parse('10')
  });

  // Bob adds initial liquidity
  await addLiquidity(test, {
    contract,
    user: bob,
    amount: NEAR.parse('50')
  });

  // Alice requests instant unstake
  await instantUnstake(test, {
    contract,
    user: alice,
    amount: NEAR.parse('5')
  });

  // Bob deposits and stakes
  await stake(test, {
    contract,
    user: bob,
    amount: NEAR.parse('3')
  });
  test.is(
    (await getTotalStakedNEAR(contract)).toString(),
    NEAR.parse("18.04485").toString()
  );

  // Bob deposits and stakes
  await stake(test, {
    contract,
    user: bob,
    amount: NEAR.parse('4')
  });
  test.is(
    (await getTotalStakedNEAR(contract)).toString(),
    NEAR.parse("22.04485").toString()
  );
});

workspace.test('configure liquidity pool', async (test, { contract, owner, alice, bob }) => {
  // Alice deposits and stakes to avoid empty stake shares
  await stake(test, {
    contract,
    user: alice,
    amount: NEAR.parse('10')
  });

  // Bob adds initial liquidity
  await addLiquidity(test, {
    contract,
    user: bob,
    amount: NEAR.parse('50')
  });

  // Increase the treasury fee to 70%
  await owner.call(
    contract,
    'configure_liquidity_pool',
    {
      config: {
        expected_near_amount: NEAR.parse("10000").toString(),
        max_fee_bps: 300,
        min_fee_bps: 30,
        treasury_fee_bps: 7000,
      }
    }
  )

  // Alice requests instant unstake
  await instantUnstake(test, {
    contract,
    user: alice,
    amount: NEAR.parse('5')
  });

  // Bob deposits and stakes
  await stake(test, {
    contract,
    user: bob,
    amount: NEAR.parse('3')
  });
  test.is(
    (await getTotalStakedNEAR(contract)).toString(),
    NEAR.parse("18.10465").toString()   // 10 + 10 + 3 - 4.89535
  );

  // Bob deposits and stakes
  await stake(test, {
    contract,
    user: bob,
    amount: NEAR.parse('4')
  });
  test.is(
    (await getTotalStakedNEAR(contract)).toString(),
    NEAR.parse("22.10465").toString()
  );
});

workspace.test('liquidity pool misconfiguration', async (test, { contract, owner }) => {

  const ERR_NON_POSITIVE_MIN_FEE = "The min fee basis points should be positive";
  const ERR_FEE_MAX_LESS_THAN_MIN = "The max fee basis points should be no less than the min fee";
  const ERR_FEE_EXCEEDS_UP_LIMIT = "The fee basis points should be less than 10000";
  const ERR_NON_POSITIVE_EXPECTED_NEAR_AMOUNT = "The expected NEAR amount should be positive";

  await assertFailure(
    test,
    owner.call(
      contract,
      'configure_liquidity_pool',
      {
        config: {
          expected_near_amount: NEAR.parse("10000").toString(),
          max_fee_bps: 300,
          min_fee_bps: 0,
          treasury_fee_bps: 3000,
        }
      }
    ),
    ERR_NON_POSITIVE_MIN_FEE
  );

  await assertFailure(
    test,
    owner.call(
      contract,
      'configure_liquidity_pool',
      {
        config: {
          expected_near_amount: NEAR.parse("10000").toString(),
          max_fee_bps: 30,
          min_fee_bps: 300,
          treasury_fee_bps: 3000,
        }
      }
    ),
    ERR_FEE_MAX_LESS_THAN_MIN
  );

  await assertFailure(
    test,
    owner.call(
      contract,
      'configure_liquidity_pool',
      {
        config: {
          expected_near_amount: NEAR.parse("10000").toString(),
          max_fee_bps: 300,
          min_fee_bps: 30,
          treasury_fee_bps: 10001,
        }
      }
    ),
    ERR_FEE_EXCEEDS_UP_LIMIT
  );

  await assertFailure(
    test,
    owner.call(
      contract,
      'configure_liquidity_pool',
      {
        config: {
          expected_near_amount: NEAR.parse("0").toString(),
          max_fee_bps: 300,
          min_fee_bps: 30,
          treasury_fee_bps: 3000,
        }
      }
    ),
    ERR_NON_POSITIVE_EXPECTED_NEAR_AMOUNT
  );

});
