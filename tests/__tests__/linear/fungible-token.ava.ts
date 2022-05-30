import { NEAR, NearAccount, BN, Gas } from 'near-workspaces-ava';
import {
  initWorkSpace,
  assertFailure,
  registerFungibleTokenUser,
  ONE_YOCTO,
  epochHeightFastforward,
  deployDex,
  skip,
} from './helper';

const ERR_NO_ENOUGH_BALANCE = 'The account doesn\'t have enough balance';
const ERR_UNREGISTER_POSITIVE_UNSTAKED =
  "Cannot delete the account because the unstaked amount is not empty. Withdraw your balance first.";
const ERR_UNREGISTER_WITH_BALANCE = "Can\'t unregister the account with the positive balance without force";

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
      amount,
    },
    {
      attachedDeposit: ONE_YOCTO
    }
  );
}

async function transferCall(
  contract: NearAccount,
  sender: NearAccount,
  receiver: NearAccount,
  amount: NEAR,
  msg: String,
  memo?: String,
) {
  await sender.call(
    contract,
    'ft_transfer_call',
    {
      receiver_id: receiver,
      amount,
      memo,
      msg,
    },
    {
      gas: Gas.parse('75 Tgas'),
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

// Ensure LiNEAR transfer work well with NEAR Wallet
workspace.test('register LiNEAR with 0.00125Ⓝ storage balance', async (test, {contract, alice, bob}) => {
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

skip('storage unregister', async (test, {contract, alice, bob}) => {
  await registerFungibleTokenUser(contract, alice);
  await registerFungibleTokenUser(contract, bob);

  test.is(
    (await contract.view('storage_balance_of', { account_id: alice }) as any).total,
    NEAR.parse("0.00125").toString()
  );

  // Unregister Alice
  await alice.call(
    contract,
    'storage_unregister',
    {},
    { attachedDeposit: ONE_YOCTO }
  );
  test.is(
    await contract.view('storage_balance_of', { account_id: alice }),
    null
  );

  // Alice deposit and stake 10 NEAR
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: NEAR.parse('10') },
  );

  // Force unregister Alice successfully.
  // The $LiNEAR owned by Alice are all burnt. Now $LiNEAR price increased to 2 $NEAER.
  await alice.call(
    contract,
    'storage_unregister',
    { force: true },
    { attachedDeposit: ONE_YOCTO }
  );
  test.is(
    await contract.view('storage_balance_of', { account_id: alice }),
    null
  );
  test.is(
    await contract.view('ft_balance_of', { account_id: alice }),
    '0',
  );

  // Alice deposit and stake 10 NEAR
  const stakeAmount = NEAR.parse('10');
  const ft_price = NEAR.from((await contract.view('ft_price', {}))).div(NEAR.parse('1'));
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: stakeAmount },
  );
  test.is(
    await contract.view('ft_balance_of', { account_id: alice }),
    stakeAmount.div(ft_price).toString(),   // 5 $LiNEAR
  );

  // transfer 1 LiNEAR from alice to bob
  await transfer(contract, alice, bob, NEAR.parse('1'));

  // Alice unstakes 2 NEAR
  await alice.call(
    contract,
    'unstake',
    { amount: NEAR.parse('2') },
  );

  // Unregister Alice when unstaked is non-zero, should fail
  await assertFailure(
    test,
    alice.call(
      contract,
      'storage_unregister',
      { force: true },
      { attachedDeposit: ONE_YOCTO }
    ),
    ERR_UNREGISTER_POSITIVE_UNSTAKED
  );

  // 4 epoches later, Alice withdraws 2 NEAR
  await epochHeightFastforward(contract, alice);
  await alice.call(
    contract,
    'withdraw',
    { amount: NEAR.parse('2') },
  );

  // non-force unregister when Alice has some LiNEAR, should fail
  await assertFailure(
    test,
    alice.call(
      contract,
      'storage_unregister',
      {},
      { attachedDeposit: ONE_YOCTO }
    ),
    ERR_UNREGISTER_WITH_BALANCE
  );

  // transfer 3 LiNEAR from alice to bob
  await transfer(contract, alice, bob, NEAR.parse('3'));

  // Now Alice could unregister successfully
  await alice.call(
    contract,
    'storage_unregister',
    {},
    { attachedDeposit: ONE_YOCTO }
  );
  test.is(
    await contract.view('storage_balance_of', { account_id: alice }),
    null
  );
});

workspace.test('ft_transfer_call LiNEAR', async (test, { root, contract, alice }) => {
  // Deploy the decentralized exchange
  const dex = await deployDex(root);
  await registerFungibleTokenUser(contract, alice, NEAR.parse("0.00125"));
  await registerFungibleTokenUser(contract, dex, NEAR.parse("0.00125"));

  // deposit and stake 10 NEAR
  const stakeAmount = NEAR.parse('10');
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: stakeAmount },
  );

  // ft_transfer_call() with `ft_on_trasfer()` passed
  const transferAmount1 = NEAR.parse('1');
  await transferCall(contract, alice, dex, transferAmount1, 'pass', 'keep my money');
  test.is(
    await contract.view('ft_balance_of', { account_id: alice }),
    stakeAmount.sub(transferAmount1).toString()
  );

  // ft_transfer_call() with `ft_on_trasfer()` failed, all assets refunded
  const transferAmount2 = NEAR.parse('2');
  await transferCall(contract, alice, dex, transferAmount2, 'fail', 'pay me 1B $NEAR');
  test.is(
    await contract.view('ft_balance_of', { account_id: alice }),
    stakeAmount.sub(transferAmount1).toString()
  );

  // ft_transfer_call() with `ft_on_trasfer()` refunded, all assets refunded
  const transferAmount3 = NEAR.parse('3');
  await transferCall(contract, alice, dex, transferAmount3, 'refund', 'refund all my assets');
  test.is(
    await contract.view('ft_balance_of', { account_id: alice }),
    stakeAmount.sub(transferAmount1).toString()
  );
});
