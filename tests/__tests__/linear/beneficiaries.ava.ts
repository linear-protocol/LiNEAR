import { assertFailure, initWorkSpace } from "./helper";

const workspace = initWorkSpace();

workspace.test('non-owner call beneficiaries', async (test, { contract, alice }) => {
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
    'Only owner can perform this action'
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
    'Only owner can perform this action'
  );
});

workspace.test('beneficiaries sum > 1', async (test, { contract, owner }) => {
  await owner.call(
    contract,
    'set_beneficiary',
    {
      account_id: 'foo',
      fraction: {
        numerator: 5,
        denominator: 10
      }
    }
  );

  await assertFailure(
    test,
    owner.call(
      contract,
      'set_beneficiary',
      {
        account_id: 'bar',
        fraction: {
          numerator: 6,
          denominator: 10
        }
      }
    ),
    'Fractions sum should be less than 1'
  );
});

workspace.test('too many beneficiaries', async (test, { contract, owner }) => {
  for (let i = 0; i < 10; i++) {
    await owner.call(
      contract,
      'set_beneficiary',
      {
        account_id: `b${i}`,
        fraction: {
          numerator: 1,
          denominator: 20
        }
      }
    );
  }

  await assertFailure(
    test,
    owner.call(
      contract,
      'set_beneficiary',
      {
        account_id: 'bar',
        fraction: {
          numerator: 1,
          denominator: 20
        }
      }
    ),
    'Too many beneficiaries'
  );
});

workspace.test('set beneficiaries', async (test, { contract, owner }) => {
  const initValues: object = await owner.call(
    contract,
    'get_beneficiaries',
    {}
  );
  test.deepEqual(initValues, {});

  await owner.call(
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
  await owner.call(
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

  const twoValues = await owner.call(
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

  await owner.call(
    contract,
    'remove_beneficiary',
    {
      account_id: 'foo'
    }
  );

  const oneValue = await owner.call(
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
