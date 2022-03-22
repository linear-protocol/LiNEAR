import { assertFailure, initWorkSpace } from "./helper";

const workspace = initWorkSpace();

workspace.test('non-admin call beneficiaries', async (test, { contract, alice }) => {
  await assertFailure(
    test,
    alice.call(
      contract,
      'set_beneficiary',
      {
        account_id: alice.accountId,
        fraction: {
          numerator: 1,
          denominator: 10
        }
      }
    ),
    'Only admin can perform this action'
  );

  await assertFailure(
    test,
    alice.call(
      contract,
      'remove_beneficiary',
      {
        account_id: alice.accountId
      }
    ),
    'Only admin can perform this action'
  );
});

workspace.test('set beneficiaries', async (test, { contract, admin }) => {
  const initValues: object = await admin.call(
    contract,
    'get_beneficiaries',
    {}
  );
  test.deepEqual(initValues, {});

  await admin.call(
    contract,
    'set_beneficiary',
    {
      account_id: 'foo',
      fraction: {
        numerator: 1,
        denominator: 10
      }
    }
  );
  await admin.call(
    contract,
    'set_beneficiary',
    {
      account_id: 'bar',
      fraction: {
        numerator: 5,
        denominator: 10
      }
    }
  );

  const twoValues = await admin.call(
    contract,
    'get_beneficiaries',
    {}
  );

  test.deepEqual(
    twoValues,
    {
      foo: {
        numerator: 1,
        denominator: 10
      },
      bar: {
        numerator: 5,
        denominator: 10
      }
    }
  );

  await admin.call(
    contract,
    'remove_beneficiary',
    {
      account_id: 'foo'
    }
  );

  const oneValue = await admin.call(
    contract,
    'get_beneficiaries',
    {}
  );

  test.deepEqual(
    oneValue,
    {
      bar: {
        numerator: 5,
        denominator: 10
      }
    }
  );
});
