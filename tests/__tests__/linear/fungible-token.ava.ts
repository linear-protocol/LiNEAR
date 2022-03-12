import { Workspace, NEAR, NearAccount } from 'near-workspaces-ava';
import {
  initWorkSpace,
  assertFailure,
  registerFungibleTokenUser,
  ONE_YOCTO
} from './helper';

const ERR_NO_ENOUGH_BALANCE = 'The account doesn\'t have enough balance';

async function transfer(
  contract: NearAccount,
  sender: NearAccount,
  receiver: NearAccount,
  amount: NEAR
) {
  await sender.call(
    contract,
    'ft_transfer',
    {
      receiver_id: receiver,
      amount:   amount.toString()
    },
    {
      attachedDeposit: ONE_YOCTO
    }
  );
}

const workspace = initWorkSpace();

workspace.test('read ft metadata', async (test, {contract, alice}) => {
  const metadata = await contract.view('ft_metadata', {}) as any;
  test.is(
    metadata.symbol,
    'LINEAR',
  );
  test.is(
    metadata.decimals,
    24
  );
});

workspace.test('ft price', async (test, {contract, alice}) => {
  const price = await contract.view('ft_price', {}) as any;
  test.is(
    NEAR.from(price).toString(),
    NEAR.parse('1').toString()
  );
});

workspace.test('cannot transfer with no balance', async (test, {contract, alice, bob}) => {
  await registerFungibleTokenUser(contract, alice);

  await assertFailure(
    test,
    transfer(contract, alice, bob, NEAR.parse('1')),
    ERR_NO_ENOUGH_BALANCE
  );
});

workspace.test('stake NEAR and transfer LiNEAR', async (test, {contract, alice, bob}) => {
  await registerFungibleTokenUser(contract, alice);
  await registerFungibleTokenUser(contract, bob);

  // deposit and stake 10 NEAR
  const stakeAmount = NEAR.parse('10');
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: stakeAmount },
  );

  // transfer 2 LiNEAR from alice to bob
  const transferAmount1 = NEAR.parse('2');
  await transfer(contract, alice, bob, transferAmount1);
  test.is(
    await contract.view('ft_balance_of', { account_id: alice }),
    stakeAmount.sub(transferAmount1).toString()
  );
  test.is(
    await contract.view('ft_balance_of', { account_id: bob }),
    transferAmount1.toString()
  );

  // transfer 1 LiNEAR from bob to alice
  const transferAmount2 = NEAR.parse('1');
  await transfer(contract, bob, alice, transferAmount2);
  test.is(
    await contract.view('ft_balance_of', { account_id: alice }),
    stakeAmount.sub(transferAmount1).add(transferAmount2).toString()
  );
  test.is(
    await contract.view('ft_balance_of', { account_id: bob }),
    transferAmount1.sub(transferAmount2).toString()
  );

  // cannot transfer 2 LiNEAR from bob
  await assertFailure(
    test,
    transfer(contract, bob, alice, NEAR.parse('2')),
    ERR_NO_ENOUGH_BALANCE
  );
});

workspace.test('transfer LiNEAR with 0.00125Ⓝ storage balance', async (test, {contract, alice, bob}) => {
  await registerFungibleTokenUser(contract, alice, NEAR.parse("0.00125"));
  await registerFungibleTokenUser(contract, bob, NEAR.parse("0.00125"));

  // deposit and stake 10 NEAR
  const stakeAmount = NEAR.parse('10');
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: stakeAmount },
  );

  // transfer 2 LiNEAR from alice to bob
  const transferAmount1 = NEAR.parse('2');
  await transfer(contract, alice, bob, transferAmount1);
  test.is(
    await contract.view('ft_balance_of', { account_id: alice }),
    stakeAmount.sub(transferAmount1).toString()
  );
  test.is(
    await contract.view('ft_balance_of', { account_id: bob }),
    transferAmount1.toString()
  );
});
