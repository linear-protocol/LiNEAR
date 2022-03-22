import { assertFailure, initWorkSpace } from "./helper";

const workspace = initWorkSpace();

workspace.test('non-owner call operator methods', async (test, { contract, alice }) => {
  await assertFailure(
    test,
    alice.call(
      contract,
      'add_operator',
      {
        new_operator_id: alice.accountId
      }
    ),
    'Only owner can perform this action'
  );

  await assertFailure(
    test,
    alice.call(
      contract,
      'remove_operator',
      {
        operator_id: alice.accountId
      },
    ),
    'Only owner can perform this action'
  );
});

workspace.test('set operator', async (test, {contract, owner, alice}) => {
  await owner.call(
    contract,
    'add_operator',
    {
      new_operator_id: alice.accountId
    }
  );

  const operators: string[] = await contract.view('get_operators');
  test.assert(operators.includes(alice.accountId));
});

workspace.test('remove operator', async (test, {contract, owner, alice}) => {
  await owner.call(
    contract,
    'add_operator',
    {
      new_operator_id: alice.accountId
    }
  );

  await owner.call(
    contract,
    'remove_operator',
    {
      operator_id: alice.accountId
    }
  );

  const operators: string[] = await contract.view('get_operators');
  test.assert(!operators.includes(alice.accountId));
});
