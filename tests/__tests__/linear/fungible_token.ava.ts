import { NEAR, } from 'near-workspaces-ava';
import { initWorkSpace } from './helper';

async function registerUser(ft: any, user: any) {
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

const workspace = initWorkSpace();

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
