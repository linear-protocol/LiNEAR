import { assertFailure, initWorkSpace } from './helper';

const workspace = initWorkSpace();

workspace.test(
  'non-owner call manager methods',
  async (test, { contract, alice }) => {
    await assertFailure(
      test,
      alice.call(contract, 'add_manager', {
        new_manager_id: alice.accountId,
      }),
      'Only owner can perform this action',
    );

    await assertFailure(
      test,
      alice.call(contract, 'remove_manager', {
        manager_id: alice.accountId,
      }),
      'Only owner can perform this action',
    );
  },
);

workspace.test('set manager', async (test, { contract, owner, alice }) => {
  await owner.call(contract, 'add_manager', {
    new_manager_id: alice.accountId,
  });

  const managers: string[] = await contract.view('get_managers');
  test.assert(managers.includes(alice.accountId));
});

workspace.test('remove manager', async (test, { contract, owner, alice }) => {
  await owner.call(contract, 'add_manager', {
    new_manager_id: alice.accountId,
  });

  await owner.call(contract, 'remove_manager', {
    manager_id: alice.accountId,
  });

  const managers: string[] = await contract.view('get_managers');
  test.assert(!managers.includes(alice.accountId));
});
