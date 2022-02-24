import { BN, NEAR, NearAccount } from 'near-workspaces-ava';
import { initWorkSpace, registerFungibleTokenUser } from './helper';

const ONE_YOCTO_NEAR = '1';

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
      attachedDeposit: ONE_YOCTO_NEAR
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
  quick?: boolean
) {
  const ft = await mintFungibleTokens(root, owner, "ft-1", NEAR.parse("100000000"));
  const now = new Date();
  const amount = NEAR.parse("1000000");
  const range = quick ? {
    start: 10,  // the start time must be later then the current time
    end: 10 + 1000000
  } : {
    start: 1 * 24 * 3600,   // 1 days later
    end: 101 * 24 * 3600    // 101 days later
  };
  const farm = {
    farm_id: 0,
    name: 'Farming #1',
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
  const { farm, ft } = await addFirstFarm(root, contract, owner, true);
  test.deepEqual(
    await contract.view("get_farm", { farm_id: 0 }),
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
  await sleep(2000);
  // Notice that Alice received 0.5 FT (50% of total) per seconds
  // because the default initial staked amount is 10Ⓝ.
  // However, it can be 2 or 3 seconds when comes to the next line.
  let rewards = await contract.view("get_unclaimed_reward", {
    account_id: alice,
    farm_id: farm.farm_id
  });
  test.true(
    rewards === NEAR.parse("1").toString()
    || rewards === NEAR.parse("1.5").toString()
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
      attachedDeposit: ONE_YOCTO_NEAR
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
  test.true(
    rewards === NEAR.parse("0.5").toString()
    || rewards === NEAR.parse("1").toString()
  );

  // Bob deposits and stakes
  const stakeAmount2 = NEAR.parse('20');
  await bob.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: stakeAmount2 },
  );
  // Wait 2 seconds for rewards: 1 FT token distributed per second
  await sleep(2000);
  // Notice that Bob received 0.5 FT (50% of total) per seconds
  // because the default initial staked amount is 10Ⓝ + Alice staked 10Ⓝ
  // However, it can be 2 or 3 seconds when comes to the next line.
  rewards = await contract.view("get_unclaimed_reward", {
    account_id: bob,
    farm_id: farm.farm_id
  });
  test.true(
    rewards === NEAR.parse("1").toString()
    || rewards === NEAR.parse("1.5").toString()
  );
});

workspace.test('stop farm', async (test, {root, contract, owner, alice, bob}) => {
  // Add farm which will start in 10s
  const { farm } = await addFirstFarm(root, contract, owner, true);
  test.deepEqual(
    await contract.view("get_farm", { farm_id: 0 }),
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
  // Wait 5 seconds for rewards: 1 FT token distributed per second
  await sleep(2000);
  // Notice that Alice received 0.5 FT (50% of total) per seconds
  // because the default initial staked amount is 10Ⓝ
  // However, it can be 2 or 3 seconds when comes to the next line.
  const rewards = await contract.view("get_unclaimed_reward", {
    account_id: alice,
    farm_id: farm.farm_id
  });
  test.true(
    rewards === NEAR.parse("1").toString()
    || rewards === NEAR.parse("1.5").toString()
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