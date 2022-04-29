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
        bps: 1000
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
      bps: 5000
    }
  );

  await assertFailure(
    test,
    owner.call(
      contract,
      'set_beneficiary',
      {
        account_id: 'bar',
        bps: 6000
      }
    ),
    'bps sum should be less than 1'
  );
});

workspace.test('too many beneficiaries', async (test, { contract, owner }) => {
  for (let i = 0; i < 10; i++) {
    await owner.call(
      contract,
      'set_beneficiary',
      {
        account_id: `b${i}`,
        bps: 100
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
        bps: 100
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
      bps: 1000
    }
  );
  await owner.call(
    contract,
    'set_beneficiary',
    {
      account_id: 'bar',
      bps: 5000
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
      foo: 1000,
      bar: 5000
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
      bar: 5000
    }
  );
});
