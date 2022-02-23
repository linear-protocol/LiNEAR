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
  const msg = JSON.stringify({
    name,
    start_date,
    end_date
  })
  await transferCall(ft, owner, contract, amount, msg);
}

workspace.test('add farm', async (test, {root, contract, owner, alice}) => {
  const ft = await mintFungibleTokens(root, owner, "ft-1", NEAR.parse("100000000"));
  await owner.call(
    contract,
    'add_authorized_farm_token',
    {
      token_id: ft
    }
  );
  const now = new Date();
  const amount = NEAR.parse("1000000");
  const farm = {
    farm_id: 0,
    name: 'Farming #1',
    token_id: ft.accountId,
    amount: amount.toString(),
    start_date: secondsLater(now, 1 * 24 * 3600),   // 1 days later
    end_date: secondsLater(now, 101 * 24 * 3600),  // end date: 101 days later
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
  test.deepEqual(
    await contract.view("get_farm", { farm_id: 0 }),
    farm
  )
});

workspace.test('stake and farm tokens', async (test, {contract, alice}) => {
  // deposit and stake
  const stakeAmount = NEAR.parse('10');
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: stakeAmount },
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

workspace.test('stop farm', async (test, {contract, alice}) => {

});