import { assertFailure, initWorkSpace } from "./helper";

const workspace = initWorkSpace();

workspace.test('non-admin call manager methods', async (test, { contract, alice }) => {
  await assertFailure(
    test,
    alice.call(
      contract,
      'add_manager',
      {
        new_manager_id: alice.accountId
      }
    ),
    'Only admin can perform this action'
  );

  await assertFailure(
    test,
    alice.call(
      contract,
      'remove_manager',
      {
        manager_id: alice.accountId
      },
    ),
    'Only admin can perform this action'
  );
});

workspace.test('set manager', async (test, {contract, admin, alice}) => {
  await admin.call(
    contract,
    'add_manager',
    {
      new_manager_id: alice.accountId
    }
  );

  const managers: string[] = await contract.view('get_managers');
  test.assert(managers.includes(alice.accountId));
});

workspace.test('remove manager', async (test, {contract, admin, alice}) => {
  await admin.call(
    contract,
    'add_manager',
    {
      new_manager_id: alice.accountId
    }
  );

  await admin.call(
    contract,
    'remove_manager',
    {
      manager_id: alice.accountId
    }
  );

  const managers: string[] = await contract.view('get_managers');
  test.assert(!managers.includes(alice.accountId));
});
