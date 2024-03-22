import { Workspace, NEAR, NearAccount } from 'near-workspaces-ava';

const ONE_YOCTO_NEAR = '1';

const workspace = Workspace.init(async ({ root }) => {
  const alice = await root.createSubAccount('alice');
  const bob = await root.createSubAccount('bob');

  const contract = await root.createAndDeploy(
    'mock-fungible-token',
    'compiled-contracts/mock_fungible_token.wasm',
    {
      method: 'new',
      args: {},
    },
  );

  return { contract, alice, bob };
});

async function registerUser(ft: NearAccount, user: NearAccount) {
  const storage_balance = (await ft.view('storage_balance_bounds', {})) as any;

  await user.call(
    ft,
    'storage_deposit',
    { account_id: user },
    // Deposit pulled from ported sim test
    { attachedDeposit: storage_balance.min.toString() },
  );
}

async function mint(contract: NearAccount, account: NearAccount, amount: NEAR) {
  await account.call(contract, 'mint', {
    account_id: account,
    amount: amount.toString(),
  });
}

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
      amount: amount.toString(),
    },
    {
      attachedDeposit: ONE_YOCTO_NEAR,
    },
  );
}

workspace.test('mint token', async (test, { contract, alice, bob }) => {
  const mintedAmount = NEAR.parse('100');
  await mint(contract, alice, mintedAmount);
  test.is(
    await contract.view('ft_balance_of', { account_id: alice }),
    mintedAmount.toString(),
  );
});

workspace.test('transfer token', async (test, { contract, alice, bob }) => {
  const mintedAmount = NEAR.parse('100');
  await mint(contract, alice, mintedAmount);
  test.is(
    await contract.view('ft_balance_of', { account_id: alice }),
    mintedAmount.toString(),
  );

  await registerUser(contract, bob);

  // transfer 10 token from alice to bob
  const transferAmount1 = NEAR.parse('10');
  await transfer(contract, alice, bob, transferAmount1);
  test.is(
    await contract.view('ft_balance_of', { account_id: alice }),
    mintedAmount.sub(transferAmount1).toString(),
  );
  test.is(
    await contract.view('ft_balance_of', { account_id: bob }),
    transferAmount1.toString(),
  );

  // transfer 5 token from bob to alice
  const transferAmount2 = NEAR.parse('5');
  await transfer(contract, bob, alice, transferAmount2);
  test.is(
    await contract.view('ft_balance_of', { account_id: alice }),
    mintedAmount.sub(transferAmount1).add(transferAmount2).toString(),
  );
  test.is(
    await contract.view('ft_balance_of', { account_id: bob }),
    transferAmount1.sub(transferAmount2).toString(),
  );
});
