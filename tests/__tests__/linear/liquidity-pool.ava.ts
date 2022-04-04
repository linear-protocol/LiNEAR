import { NEAR, BN, Gas } from 'near-workspaces-ava';
import {
  initWorkSpace,
  callWithMetrics,
  numbersEqual,
  noMoreThanOneYoctoDiff,
  assertFailure,
  ONE_YOCTO
} from './helper';

// Errors
const ERR_NON_POSITIVE_REMOVE_LIQUIDITY_AMOUNT = "The amount of value to be removed from liquidity pool should be positive";

// helper functions

let config = null;
const getConfig = async (contract) => {
  if (!config) {
    config = await contract.view("get_liquidity_pool_config", {}) as any;
    config = {
      ...config,
      max_fee_bps: new BN(Number(config.max_fee_bps)),
      min_fee_bps: new BN(Number(config.min_fee_bps)),
      expected_near_amount: NEAR.from(config.expected_near_amount),
    }
  }
  return config;
}

// Estimate swap fee
const estimateSwapFee = async (contract: any, totalAmount: NEAR, amount: NEAR) => {
  let { expected_near_amount, max_fee_bps, min_fee_bps } = await getConfig(contract);
  let diff = max_fee_bps.sub(min_fee_bps);
  const remainingLiquidity = totalAmount.sub(amount);
  if (remainingLiquidity.gt(expected_near_amount)) {
    return amount.mul(min_fee_bps).div(new BN(10000));
  } else {
    return amount.mul(
      max_fee_bps.sub(
        diff.mul(remainingLiquidity).div(expected_near_amount)
      )
    ).div(new BN(10000));
  }
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
  const { lp_near_amount, lp_staked_share } = summary;
  return NEAR.from(lp_near_amount).add(
    await stakeSharesValues(contract, NEAR.from(lp_staked_share))
  );
}

const getPoolAccountValue = async (contract, account) => {
  return NEAR.from((await contract.view('get_account_info', { account_id: account }) as any).liquidity_pool_share_value);
}

const stakeSharesValues = async (contract, stake_shares: NEAR) => {
  return stake_shares.mul(NEAR.from(await contract.view('ft_price', {}))).div(NEAR.parse('1'));
}

const stake = async(test, {contract, user, amount}) => {
  await user.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: amount },
  );
}

const unstake = async(test, {contract, user, amount}) => {
  await user.call(
    contract,
    'unstake',
    { amount },
  );
}

const addLiquidity = async(test, {contract, user, amount}) => {
  const previousPoolValue = await getPoolValue(contract);
  const prevAccountPoolValue = await getPoolAccountValue(contract, user);
  await user.call(
    contract,
    'add_liquidity',
    {},
    { attachedDeposit: amount },
  );
  const updatedPoolValue = await getPoolValue(contract);
  const updatedAccountPoolValue = await getPoolAccountValue(contract, user);
  test.is(
    prevAccountPoolValue.add(amount).toString(),
    updatedAccountPoolValue.toString()
  );
  test.is(
    previousPoolValue.add(amount).toString(),
    updatedPoolValue.toString()
  );
}

const removeLiquidity = async (test, {contract, user, amount, loss = "1"}) => {
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
  const receivedStakedShareValue = await stakeSharesValues(contract, receivedStakedShare);
  noMoreThanOneYoctoDiff(
    test,
    receivedAmount.add(receivedStakedShareValue),
    amount,
    loss
  );
  noMoreThanOneYoctoDiff(
    test,
    updatedPoolValue,
    previousPoolValue.sub(amount),
    loss
  );
  // Fuzzy match due to balance accuracy issue
  numbersEqual(
    test,
    balance.add(receivedAmount).sub(result.metrics.tokensBurnt),
    await getBalance(user),
    0.025
  );
}

const instantUnstake = async (test, {contract, user, amount}) => {
  const summary = await getSummary(contract);
  const nearAmount = await stakeSharesValues(contract, amount);
  const totalAmount = NEAR.from((summary as any).lp_near_amount);
  let fee = await estimateSwapFee(contract, totalAmount, nearAmount);
  const receivedAmount: string = await user.call(
    contract,
    'instant_unstake',
    {
      stake_shares_in: amount.toString(),
      min_amount_out: nearAmount.sub(fee)
        .mul(new BN(9900)).div(new BN(10000)).toString()
    },
    {
      gas: Gas.parse('50 Tgas')
    }
  );
  noMoreThanOneYoctoDiff(
    test,
    NEAR.from(receivedAmount),
    nearAmount.sub(fee),
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
  // Bob removes 0 liquidity
  await assertFailure(
    test,
    removeLiquidity(test, {
      contract,
      user: bob,
      amount: NEAR.parse('0')
    }),
    ERR_NON_POSITIVE_REMOVE_LIQUIDITY_AMOUNT
  );

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
  await removeLiquidity(test, {
    contract,
    user: bob,
    amount: NEAR.parse('10')
  });

  // Bob removes liquidity from pool for the 2nd time
  await removeLiquidity(test, {
    contract,
    user: bob,
    amount: NEAR.parse('5')
  });

  // Bob removes 0 liquidity
  await assertFailure(
    test,
    removeLiquidity(test, {
      contract,
      user: bob,
      amount: NEAR.parse('0')
    }),
    ERR_NON_POSITIVE_REMOVE_LIQUIDITY_AMOUNT
  );
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
  await removeLiquidity(test, {
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
  await removeLiquidity(test, {
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

workspace.test('issue: panick if remove account total liquidity (LiNEAR price > 1.0, liquidity > 10K)',
  async (test, { contract, owner, alice, bob, carol }) => {

  // Alice deposits and stakes to avoid empty stake shares
  await stake(test, {
    contract,
    user: alice,
    amount: NEAR.parse('40')
  });

  // Add 0.5N epoch rewards, pirce becomes 1.01
  await owner.call(
    contract,
    'add_epoch_rewards',
    { amount: NEAR.parse('0.5') }
  );
  test.is(
    await contract.view('ft_price'),
    NEAR.parse('1.01').toString()
  );

  // Bob adds liquidity
  await addLiquidity(test, {
    contract,
    user: bob,
    amount: NEAR.parse('100000')
  });

  // Alice deposits and stakes
  await stake(test, {
    contract,
    user: alice,
    amount: NEAR.parse('5000')
  });

  // Alice delayed unstake
  await unstake(test, {
    contract,
    user: alice,
    amount: NEAR.parse('1250')
  });

  // Alice instant unstake
  await instantUnstake(test, {
    contract,
    user: alice,
    amount: NEAR.parse('1250')
  });

  // Alice adds liquidity
  await addLiquidity(test, {
    contract,
    user: alice,
    amount: NEAR.parse('50')
  });

  // Alice removes liquidity
  await removeLiquidity(test, {
    contract,
    user: alice,
    amount: NEAR.parse('5')
  });

  // Add 100N epoch rewards
  await owner.call(
    contract,
    'add_epoch_rewards',
    { amount: NEAR.parse('100') }
  );

  // Carol adds liquidity
  await addLiquidity(test, {
    contract,
    user: carol,
    amount: NEAR.parse('10')
  });

  // Carol removes liquidity
  await removeLiquidity(test, {
    contract,
    user: carol,
    amount: NEAR.parse('10'),
    // The loss is higher since rounded up is not possible which will exceeds the
    // account's total shares
    loss: '3' // yoctoN
  });
});
