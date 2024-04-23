import { NEAR, NearAccount, BN, Gas } from 'near-workspaces';
import {
  initWorkspace,
  assertFailure,
  registerFungibleTokenUser,
  ONE_YOCTO,
  epochHeightFastforward,
  deployDex,
  test,
} from './helper';

const ERR_NO_ENOUGH_BALANCE = "The account doesn't have enough balance";
const ERR_UNREGISTER_POSITIVE_UNSTAKED =
  'Cannot delete the account because the unstaked amount is not empty. Withdraw your balance first.';
const ERR_UNREGISTER_WITH_BALANCE =
  "Can't unregister the account with the positive balance without force";

async function transfer(
  contract: NearAccount,
  sender: NearAccount,
  receiver: NearAccount,
  amount: NEAR,
) {
  await sender.call(
    contract,
    'ft_transfer',
    {
      receiver_id: receiver,
      amount,
    },
    {
      attachedDeposit: ONE_YOCTO,
    },
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
      attachedDeposit: ONE_YOCTO,
    },
  );
}

test.beforeEach(async (t) => {
  t.context = await initWorkspace();
});

test.afterEach(async (t) => {
  await t.context.worker.tearDown();
});

test('read ft metadata', async (t) => {
  const { contract, alice } = t.context;
  const metadata = (await contract.view('ft_metadata', {})) as any;
  t.is(metadata.symbol, 'LINEAR');
  t.is(metadata.decimals, 24);
});

test('ft price', async (t) => {
  const { contract } = t.context;
  const price = (await contract.view('ft_price', {})) as any;
  t.is(NEAR.from(price).toString(), NEAR.parse('1').toString());
});

test('cannot transfer with no balance', async (t) => {
  const { contract, alice, bob } = t.context;
  await registerFungibleTokenUser(contract, alice);

  await assertFailure(
    t,
    transfer(contract, alice, bob, NEAR.parse('1')),
    ERR_NO_ENOUGH_BALANCE,
  );
});

test('stake NEAR and transfer LiNEAR', async (t) => {
  const { contract, alice, bob } = t.context;
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
  t.is(
    await contract.view('ft_balance_of', { account_id: alice }),
    stakeAmount.sub(transferAmount1).toString(),
  );
  t.is(
    await contract.view('ft_balance_of', { account_id: bob }),
    transferAmount1.toString(),
  );

  // transfer 1 LiNEAR from bob to alice
  const transferAmount2 = NEAR.parse('1');
  await transfer(contract, bob, alice, transferAmount2);
  t.is(
    await contract.view('ft_balance_of', { account_id: alice }),
    stakeAmount.sub(transferAmount1).add(transferAmount2).toString(),
  );
  t.is(
    await contract.view('ft_balance_of', { account_id: bob }),
    transferAmount1.sub(transferAmount2).toString(),
  );

  // cannot transfer 2 LiNEAR from bob
  await assertFailure(
    t,
    transfer(contract, bob, alice, NEAR.parse('2')),
    ERR_NO_ENOUGH_BALANCE,
  );
});

// Ensure LiNEAR transfer work well with NEAR Wallet
test('register LiNEAR with 0.00125Ⓝ storage balance', async (t) => {
  const { contract, alice, bob } = t.context;
  await registerFungibleTokenUser(contract, alice, NEAR.parse('0.00125'));
  await registerFungibleTokenUser(contract, bob, NEAR.parse('0.00125'));

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
  t.is(
    await contract.view('ft_balance_of', { account_id: alice }),
    stakeAmount.sub(transferAmount1).toString(),
  );
  t.is(
    await contract.view('ft_balance_of', { account_id: bob }),
    transferAmount1.toString(),
  );
});

test.skip('storage unregister', async (t) => {
  const { contract, alice, bob } = t.context;
  await registerFungibleTokenUser(contract, alice);
  await registerFungibleTokenUser(contract, bob);

  t.is(
    ((await contract.view('storage_balance_of', { account_id: alice })) as any)
      .total,
    NEAR.parse('0.00125').toString(),
  );

  // Unregister Alice
  await alice.call(
    contract,
    'storage_unregister',
    {},
    { attachedDeposit: ONE_YOCTO },
  );
  t.is(await contract.view('storage_balance_of', { account_id: alice }), null);

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
    { attachedDeposit: ONE_YOCTO },
  );
  t.is(await contract.view('storage_balance_of', { account_id: alice }), null);
  t.is(await contract.view('ft_balance_of', { account_id: alice }), '0');

  // Alice deposit and stake 10 NEAR
  const stakeAmount = NEAR.parse('10');
  const ft_price = NEAR.from(await contract.view('ft_price', {})).div(
    NEAR.parse('1'),
  );
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    { attachedDeposit: stakeAmount },
  );
  t.is(
    await contract.view('ft_balance_of', { account_id: alice }),
    stakeAmount.div(ft_price).toString(), // 5 $LiNEAR
  );

  // transfer 1 LiNEAR from alice to bob
  await transfer(contract, alice, bob, NEAR.parse('1'));

  // Alice unstakes 2 NEAR
  await alice.call(contract, 'unstake', { amount: NEAR.parse('2') });

  // Unregister Alice when unstaked is non-zero, should fail
  await assertFailure(
    t,
    alice.call(
      contract,
      'storage_unregister',
      { force: true },
      { attachedDeposit: ONE_YOCTO },
    ),
    ERR_UNREGISTER_POSITIVE_UNSTAKED,
  );

  // 4 epoches later, Alice withdraws 2 NEAR
  await epochHeightFastforward(contract, alice);
  await alice.call(contract, 'withdraw', { amount: NEAR.parse('2') });

  // non-force unregister when Alice has some LiNEAR, should fail
  await assertFailure(
    t,
    alice.call(
      contract,
      'storage_unregister',
      {},
      { attachedDeposit: ONE_YOCTO },
    ),
    ERR_UNREGISTER_WITH_BALANCE,
  );

  // transfer 3 LiNEAR from alice to bob
  await transfer(contract, alice, bob, NEAR.parse('3'));

  // Now Alice could unregister successfully
  await alice.call(
    contract,
    'storage_unregister',
    {},
    { attachedDeposit: ONE_YOCTO },
  );
  t.is(await contract.view('storage_balance_of', { account_id: alice }), null);
});

test('ft_transfer_call LiNEAR', async (t) => {
  const { root, contract, alice } = t.context;
  // Deploy the decentralized exchange
  const dex = await deployDex(root);
  await registerFungibleTokenUser(contract, alice, NEAR.parse('0.00125'));
  await registerFungibleTokenUser(contract, dex, NEAR.parse('0.00125'));

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
  await transferCall(
    contract,
    alice,
    dex,
    transferAmount1,
    'pass',
    'keep my money',
  );
  t.is(
    await contract.view('ft_balance_of', { account_id: alice }),
    stakeAmount.sub(transferAmount1).toString(),
  );

  // ft_transfer_call() with `ft_on_trasfer()` failed, all assets refunded
  const transferAmount2 = NEAR.parse('2');
  await transferCall(
    contract,
    alice,
    dex,
    transferAmount2,
    'fail',
    'pay me 1B $NEAR',
  );
  t.is(
    await contract.view('ft_balance_of', { account_id: alice }),
    stakeAmount.sub(transferAmount1).toString(),
  );

  // ft_transfer_call() with `ft_on_trasfer()` refunded, all assets refunded
  const transferAmount3 = NEAR.parse('3');
  await transferCall(
    contract,
    alice,
    dex,
    transferAmount3,
    'refund',
    'refund all my assets',
  );
  t.is(
    await contract.view('ft_balance_of', { account_id: alice }),
    stakeAmount.sub(transferAmount1).toString(),
  );
});
