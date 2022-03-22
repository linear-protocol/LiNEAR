import { BN, NEAR, NearAccount } from 'near-workspaces-ava';
import {
  initWorkSpace,
  registerFungibleTokenUser,
  matchMultipleValues,
  ONE_YOCTO
} from './helper';

const workspace = initWorkSpace();

async function mintFungibleTokens (
  root: NearAccount,
  account: NearAccount,
  id: string,
  amount: NEAR
) {
  const contract = await root.createAndDeploy(
    id,
    'compiled-contracts/mock_fungible_token.wasm',
    {
      method: 'new',
      args: {}
    }
  );
  // mint tokens
  await root.call(
    contract,
    'mint',
    {
      account_id: account,
      amount: amount.toString()
    }
  );
  return contract;
}

async function transferCall(
  contract: NearAccount,
  sender: NearAccount,
  receiver: NearAccount,
  amount: NEAR,
  msg: string
) {
  await registerFungibleTokenUser(contract, receiver);
  await sender.call(
    contract,
    'ft_transfer_call',
    {
      receiver_id: receiver,
      amount: amount.toString(),
      msg
    },
    {
      gas: new BN("50000000000000"),
      attachedDeposit: ONE_YOCTO
    }
  );
}

function dateToTimestamp(date: Date) {
  return date.getTime() + '000000'
}

function secondsLater(now: Date, seconds: number) {
  return dateToTimestamp(new Date(now.getTime() + seconds * 1000));
}

async function addFarm(
  contract: NearAccount,
  owner: NearAccount,
  ft: NearAccount,
  name: string,
  amount: NEAR,
  start_date: string,
  end_date: string,
) {
  await owner.call(
    contract,
    'add_authorized_farm_token',
    {
      token_id: ft
    }
  );
  const msg = JSON.stringify({
    name,
    start_date,
    end_date
  })
  await transferCall(ft, owner, contract, amount, msg);
}

async function addFirstFarm(
  root: NearAccount,
  contract: NearAccount,
  owner: NearAccount,
  type?: number
) {
  const ft = await mintFungibleTokens(root, owner, "ft-1", NEAR.parse("100000000")); // 100M
  const now = new Date();
  const amount = NEAR.parse("1000000"); // 1M
  const range = type === 1 ? {
    start: 10,  // the start time must be later then the current time
    end: 10 + 1000000
  } : ( type === 2 ? {
    start: 10,
    end: 10 + 20  // end in 10s
  } : {
    start: 1 * 24 * 3600,   // 1 days later
    end: 101 * 24 * 3600    // 101 days later
  });
  const farm = {
    farm_id: 0,
    name: 'Farm #1',
    token_id: ft.accountId,
    amount: amount.toString(),
    start_date: secondsLater(now, range.start),
    end_date: secondsLater(now, range.end),
    active: true
  }
  await addFarm(
    contract,
    owner,
    ft, 
    farm.name,
    amount,
    farm.start_date,
    farm.end_date
  );
  return { farm, ft } ;
}

async function addSecondFarm(
  root: NearAccount,
  contract: NearAccount,
  owner: NearAccount,
  quick?: boolean
) {
  const ft = await mintFungibleTokens(root, owner, "ft-2", NEAR.parse("1000000000")); // 1B
  const now = new Date();
  const amount = NEAR.parse("5000000"); // 5M
  const range = quick ? {
    start: 10,  // the start time must be later then the current time
    end: 10 + 1250000
  } : {
    start: 1 * 24 * 3600,   // 1 days later
    end: 366 * 24 * 3600    // 366 days later
  };
  const farm = {
    farm_id: 1,
    name: 'Farm #2',
    token_id: ft.accountId,
    amount: amount.toString(),
    start_date: secondsLater(now, range.start),
    end_date: secondsLater(now, range.end),
    active: true
  }
  await addFarm(
    contract,
    owner,
    ft, 
    farm.name,
    amount,
    farm.start_date,
    farm.end_date
  );
  return { farm, ft } ;
}

function sleep(ms: number) {
  return new Promise( resolve => setTimeout(resolve, ms) );
}

function delayedRewards(rewards: number, timeElapsed: number, delayedMs: number) {
  return rewards * (timeElapsed + delayedMs) / timeElapsed;
}

function assertUnclaimedRewards(
  test: any,
  actual: any,
  expected: number,
  timeElapsed: number
) {
  // Extra 1 or 2 seconds might have passed when we get the unclaimed rewards
  const rewardsDelay1s = delayedRewards(expected, timeElapsed, 1000);
  const rewardsDelay2s = delayedRewards(expected, timeElapsed, 2000);
  matchMultipleValues(
    test,
    actual as string,
    [
      NEAR.parse(expected.toString()).toString(),
      NEAR.parse(rewardsDelay1s.toString()).toString(),
      NEAR.parse(rewardsDelay2s.toString()).toString(),
    ]
  );
}


// Please notice the staking farm feature is time-sensitive.
// In the test cases, we added few `sleep(ms)` to wait for rewards being distributed,
// but this brings some uncertainty to the rewards amount because the execution
// time of contract call may vary. So we validate the rewards amount as long as its
// value is within the expected range.

workspace.test('add farm', async (test, {root, contract, owner}) => {
  // Add farm which will start one day later
  const { farm } = await addFirstFarm(root, contract, owner);
  test.deepEqual(
    await contract.view("get_farm", { farm_id: farm.farm_id }),
    farm
  );
});

workspace.test('stake and receive rewards', async (test, {root, contract, owner, alice, bob}) => {
  // Add farm which will start in 10s
  const { farm, ft } = await addFirstFarm(root, contract, owner, 1);
  test.deepEqual(
    await contract.view("get_farm", { farm_id: farm.farm_id }),
    farm
  );
  // Wait until farm starts
  await sleep(10000);

  // Alice deposits and stakes
  const stakeAmount = NEAR.parse('10');
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: stakeAmount },
  );
  // Wait 2 seconds for rewards: 1 FT token distributed per second
  let timeElapsed = 2000;
  await sleep(timeElapsed);
  // Notice that Alice received 0.5 FT (50% of total) per second
  // because the default initial staked amount is 10Ⓝ.
  // However, it can be 2 or 3 seconds later when comes to the next line.
  let rewards = await contract.view("get_unclaimed_reward", {
    account_id: alice,
    farm_id: farm.farm_id
  });
  assertUnclaimedRewards(
    test,
    rewards,
    1,
    timeElapsed
  );

  // Register Alice for FT, otherwise claim will fail
  await registerFungibleTokenUser(ft, alice);
  // Alice claims FT rewards, and check FT balance
  await alice.call(
    contract,
    'claim',
    { token_id: ft },
    {
      gas: new BN("75000000000000"), 
      attachedDeposit: ONE_YOCTO
    },
  );
  test.true(
    NEAR.from(await ft.view('ft_balance_of', {
      account_id: alice
    })).gt(NEAR.parse('1'))
  );
  // Alice has fewer unclaimed rewards now
  rewards = await contract.view("get_unclaimed_reward", {
    account_id: alice,
    farm_id: farm.farm_id
  });
  assertUnclaimedRewards(
    test,
    rewards,
    0.5,
    1000
  );

  // Next, Bob deposits and stakes
  const stakeAmount2 = NEAR.parse('20');
  await bob.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: stakeAmount2 },
  );
  // Wait 2 seconds for rewards: 1 FT token distributed per second
  timeElapsed = 2000;
  await sleep(timeElapsed);
  // Notice that Bob received 0.5 FT (50% of total) per second
  // because the default initial staked amount is 10Ⓝ + Alice staked 10Ⓝ
  // However, it can be 2 or 3 seconds later when comes to the next line.
  rewards = await contract.view("get_unclaimed_reward", {
    account_id: bob,
    farm_id: farm.farm_id
  });
  assertUnclaimedRewards(
    test,
    rewards,
    1,
    timeElapsed
  );
});

workspace.test('stop farm', async (test, {root, contract, owner, alice, bob}) => {
  // Add farm which will start in 10s
  const { farm } = await addFirstFarm(root, contract, owner, 1);
  test.deepEqual(
    await contract.view("get_farm", { farm_id: farm.farm_id }),
    farm
  );
  // Wait until farm starts
  await sleep(10000);

  // Alice deposits and stakes
  const stakeAmount = NEAR.parse('10');
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: stakeAmount },
  );
  // Wait 2 seconds for rewards: 1 FT token distributed per second
  let timeElapsed = 2000;
  await sleep(timeElapsed);
  // Notice that Alice received 0.5 FT (50% of total) per second
  // because the default initial staked amount is 10Ⓝ
  // However, it can be 2 or 3 seconds later when comes to the next line.
  const rewards = await contract.view("get_unclaimed_reward", {
    account_id: alice,
    farm_id: farm.farm_id
  });
  assertUnclaimedRewards(
    test,
    rewards,
    1,
    timeElapsed
  );

  // Stop farm
  await owner.call(
    contract,
    'stop_farm',
    { farm_id: farm.farm_id }
  );
  const finalRewards = await contract.view("get_unclaimed_reward", {
    account_id: alice,
    farm_id: farm.farm_id
  });
  // Wait 5 seconds, rewards have no change
  await sleep(5000);
  test.is(
    await contract.view("get_unclaimed_reward", {
      account_id: alice,
      farm_id: farm.farm_id
    }),
    finalRewards
  );
});

workspace.test('add two farms and receive rewards', async (test, {root, contract, owner, alice, bob}) => {
  // Add farms which will start in 10s
  const { farm: farm1, ft: ft1 } = await addFirstFarm(root, contract, owner, 1);
  test.deepEqual(
    await contract.view("get_farm", { farm_id: farm1.farm_id }),
    farm1
  );
  const { farm: farm2, ft: ft2 } = await addSecondFarm(root, contract, owner, true);
  test.deepEqual(
    await contract.view("get_farm", { farm_id: farm2.farm_id }),
    farm2
  );

  // Wait until farm starts
  await sleep(10000);

  // Alice deposits and stakes
  const stakeAmount = NEAR.parse('10');
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: stakeAmount },
  );
  // Wait 2 seconds for rewards: 
  // (1) One FT-1 token distributed per second
  // (2) Four FT-2 tokens distributed per seconds
  let timeElapsed = 2000;
  await sleep(timeElapsed);
  // Alice will receive 0.5 FT-1 (50% of total) per second
  // and Alice will receive 2 FT-2 (50% of total) per second,
  // because the default initial staked amount is 10Ⓝ.
  // However, it can be 2 or 3 seconds later when comes to the next line.
  let [rewards1, rewards2] = await Promise.all([
    contract.view("get_unclaimed_reward", {
      account_id: alice,
      farm_id: farm1.farm_id
    }),
    contract.view("get_unclaimed_reward", {
      account_id: alice,
      farm_id: farm2.farm_id
    }),
  ]);
  assertUnclaimedRewards(
    test,
    rewards1,
    1,
    timeElapsed
  );
  assertUnclaimedRewards(
    test,
    rewards2,
    4,
    timeElapsed
  );

  // Register Alice for FT-1, otherwise claim will fail
  await registerFungibleTokenUser(ft1, alice);
  // Alice claims FT-1 rewards, and check FT-1 balance
  await alice.call(
    contract,
    'claim',
    { token_id: ft1 },
    {
      gas: new BN("75000000000000"), 
      attachedDeposit: ONE_YOCTO
    },
  );
  test.true(
    NEAR.from(await ft1.view('ft_balance_of', {
      account_id: alice
    })).gt(NEAR.parse('1'))
  );
  // Alice has fewer unclaimed rewards now
  rewards1 = await contract.view("get_unclaimed_reward", {
    account_id: alice,
    farm_id: farm1.farm_id
  });
  assertUnclaimedRewards(
    test,
    rewards1,
    0.5,
    1000
  );

  // Register Alice for FT-2, otherwise claim will fail
  await registerFungibleTokenUser(ft2, alice);
  // Alice claims FT-2 rewards, and check FT-2 balance
  await alice.call(
    contract,
    'claim',
    { token_id: ft2 },
    {
      gas: new BN("75000000000000"), 
      attachedDeposit: ONE_YOCTO
    },
  );
  test.true(
    NEAR.from(await ft2.view('ft_balance_of', {
      account_id: alice
    })).gt(NEAR.parse('4'))
  );
  // Alice has fewer unclaimed rewards now
  rewards2 = await contract.view("get_unclaimed_reward", {
    account_id: alice,
    farm_id: farm2.farm_id
  });
  assertUnclaimedRewards(
    test,
    rewards2,
    2,
    1000
  );

  // Next, Bob deposits and stakes
  const stakeAmount2 = NEAR.parse('20');
  await bob.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: stakeAmount2 },
  );
  // Wait 2 seconds for rewards: 
  // (1) One FT-1 token distributed per second
  // (2) Four FT-2 tokens distributed per seconds
  timeElapsed = 2000;
  await sleep(timeElapsed);
  // Bob will receive 0.5 FT-1 (50% of total) per second
  // and Bob will receive 2 FT-2 (50% of total) per second,
  // because the default initial staked amount is 10Ⓝ + Alice staked 10Ⓝ
  // However, it can be 2 or 3 seconds later when comes to the next line.
  [rewards1, rewards2] = await Promise.all([
    contract.view("get_unclaimed_reward", {
      account_id: bob,
      farm_id: farm1.farm_id
    }),
    contract.view("get_unclaimed_reward", {
      account_id: bob,
      farm_id: farm2.farm_id
    }),
  ]);
  assertUnclaimedRewards(
    test,
    rewards1,
    1,
    timeElapsed
  );
  assertUnclaimedRewards(
    test,
    rewards2,
    4,
    timeElapsed
  );
});

workspace.test('active farm has ended', async (test, {root, contract, owner, alice, bob}) => {
  // Add farm which will start in 10s
  const { farm } = await addFirstFarm(root, contract, owner, 2);
  test.deepEqual(
    await contract.view("get_farm", { farm_id: farm.farm_id }),
    farm
  );
  // Wait until farm starts
  await sleep(10000);

  // Alice deposits and stakes
  const stakeAmount = NEAR.parse('10');
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: stakeAmount },
  );
  // Wait 2 seconds for rewards: 50K FT token distributed per second
  let timeElapsed = 2000;
  await sleep(timeElapsed);
  // Notice that Alice received 25K FT (50% of total) per second
  // because the default initial staked amount is 10Ⓝ
  // However, it can be 2 or 3 seconds later when comes to the next line.
  const rewards = await contract.view("get_unclaimed_reward", {
    account_id: alice,
    farm_id: farm.farm_id
  });
  assertUnclaimedRewards(
    test,
    rewards,
    50000,
    timeElapsed
  );
  // Wait 20 seconds, check whether the farm has ended
  await sleep(20000);
  // The farm should end, but it actually needs someone to 
  // stake or unstake again to mark the farm as inactive
  test.deepEqual(
    await contract.view("get_active_farms", {}),
    [farm]
  );

  // Next, Bob deposits and stakes
  const stakeAmount2 = NEAR.parse('20');
  await bob.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: stakeAmount2 },
  );
  // The farm should end now
  test.deepEqual(
    await contract.view("get_active_farms", {}),
    []
  );
});
