import { Workspace, NEAR, NearAccount } from 'near-workspaces-ava';

const ONE_YOCTO_NEAR = '1';
const ERR_NO_ENOUGH_BALANCE = 'Smart contract panicked: The account doesn\'t have enough balance';

async function registerUser(ft: NearAccount, user: NearAccount) {
  const storage_balance = await ft.view(
    'storage_balance_bounds',
    {}
  ) as any;

  await user.call(
    ft,
    'storage_deposit',
    { account_id: user },
    // Deposit pulled from ported sim test
    { attachedDeposit: storage_balance.min.toString() },
  );
}

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
      attachedDeposit: ONE_YOCTO_NEAR
    }
  );
}

const workspace = Workspace.init(async ({root}) => {
  const owner = await root.createAccount('linear_owner');
  const alice = await root.createAccount('alice');
  const bob = await root.createAccount('bob');

  const contract = await root.createAndDeploy(
    'linear',
    'compiled-contracts/linear.wasm',
    {
      method: 'new',
      args: {
        owner_id: 'linear_owner',
        reward_fee: {
          numerator: 1,
          denominator: 100 
        }
      },
    },
  );

  return { contract, alice, bob };
});

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

workspace.test('cannot transfer with no balance', async (test, {contract, alice, bob}) => {
  await registerUser(contract, alice);

  try {
    await transfer(contract, alice, bob, NEAR.parse('1'));
  } catch(e) {
    test.is(e.kind.ExecutionError, ERR_NO_ENOUGH_BALANCE);
  }
});

workspace.test('stake NEAR and transfer LiNEAR', async (test, {contract, alice, bob}) => {
  await registerUser(contract, alice);
  await registerUser(contract, bob);

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
  try {
    await transfer(contract, bob, alice, NEAR.parse('2'));
  } catch(e) {
    test.is(e.kind.ExecutionError, ERR_NO_ENOUGH_BALANCE);
  }
});
