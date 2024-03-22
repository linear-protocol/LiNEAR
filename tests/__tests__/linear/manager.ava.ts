import { assertFailure, initWorkSpace, test } from './helper';

test.before(async (t) => {
  t.context = await initWorkSpace();
});

test.after(async (t) => {
  await t.context.worker.tearDown();
});

test(
  'non-owner call manager methods',
  async (t) => {
    const { contract, alice } = t.context;
    await assertFailure(
      t,
      alice.call(contract, 'add_manager', {
        new_manager_id: alice.accountId,
      }),
      'Only owner can perform this action',
    );

    await assertFailure(
      t,
      alice.call(contract, 'remove_manager', {
        manager_id: alice.accountId,
      }),
      'Only owner can perform this action',
    );
  },
);

test('set manager', async (t) => {
  const { contract, owner, alice } = t.context;
  await owner.call(contract, 'add_manager', {
    new_manager_id: alice.accountId,
  });

  const managers: string[] = await contract.view('get_managers');
  t.assert(managers.includes(alice.accountId));
});

test('remove manager', async (t) => {
  const { contract, owner, alice } = t.context;
  await owner.call(contract, 'add_manager', {
    new_manager_id: alice.accountId,
  });

  await owner.call(contract, 'remove_manager', {
    manager_id: alice.accountId,
  });

  const managers: string[] = await contract.view('get_managers');
  t.assert(!managers.includes(alice.accountId));
});
