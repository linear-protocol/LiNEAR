import { Workspace, NEAR, NearAccount } from 'near-workspaces-ava';

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

const workspace = Workspace.init(async ({root}) => {
  const owner = await root.createAccount('linear_owner');
  const alice = await root.createAccount('alice');

  const contract = await root.createAndDeploy(
    'linear',
    'compiled-contracts/linear.wasm',
    {
      method: 'new',
      args: {
        owner_id: 'linear_owner',
        reward_fee_fraction: {
          numerator: 1,
          denominator: 100 
        }
      },
    },
  );

  return { contract, alice };
});

workspace.test('fungible token: metadata', async (test, {contract, alice}) => {
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

// TODO: not fully tested yet; we need to call `stake()` first to get LiNEAR
workspace.test('fungible token: transfer', async (test, {contract, alice}) => {
  const ONE_YOCTO_NEAR = '1';

  await registerUser(contract, alice);

  try {
    await alice.call(
      contract,
      'ft_transfer',
      {
        receiver_id: 'bob.test.near',
        amount: NEAR.parse('1').toString()
      },
      {
        attachedDeposit: ONE_YOCTO_NEAR
      }
    );
  } catch(e) {
    test.is(
      e.kind.ExecutionError,
      'Smart contract panicked: The account doesn\'t have enough balance'
    );
  }
  
});
