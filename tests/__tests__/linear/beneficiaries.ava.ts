import { assertFailure, initWorkSpace, WorkSpace } from './helper';
import { ExecutionContext } from 'ava';
import { test } from './helper';

test.before(async (t) => {
  t.context = await initWorkSpace();
});

test.after(async (t) => {
  await t.context.worker.tearDown();
});

test('non-owner call beneficiaries', async (t: ExecutionContext<WorkSpace>) => {
  const { alice, contract } = t.context;
  await assertFailure(
    t,
    alice.call(contract, 'set_beneficiary', {
      account_id: alice.accountId,
      bps: 1000,
    }),
    'Only owner can perform this action',
  );

  await assertFailure(
    t,
    alice.call(contract, 'remove_beneficiary', {
      account_id: alice.accountId,
    }),
    'Only owner can perform this action',
  );
});

test('beneficiaries sum > 1', async (t: ExecutionContext<WorkSpace>) => {
  const { contract, owner } = t.context;
  await owner.call(contract, 'set_beneficiary', {
    account_id: 'foo',
    bps: 5000,
  });

  await assertFailure(
    t,
    owner.call(contract, 'set_beneficiary', {
      account_id: 'bar',
      bps: 6000,
    }),
    'bps sum should be less than 1',
  );
});

test('too many beneficiaries', async (t: ExecutionContext<WorkSpace>) => {
  const { contract, owner } = t.context;
  for (let i = 0; i < 10; i++) {
    await owner.call(contract, 'set_beneficiary', {
      account_id: `b${i}`,
      bps: 100,
    });
  }

  await assertFailure(
    t,
    owner.call(contract, 'set_beneficiary', {
      account_id: 'bar',
      bps: 100,
    }),
    'Too many beneficiaries',
  );
});

test('set beneficiaries', async (t: ExecutionContext<WorkSpace>) => {
  const { contract, owner } = t.context;
  const initValues: object = await owner.call(
    contract,
    'get_beneficiaries',
    {},
  );
  t.deepEqual(initValues, {});

  await owner.call(contract, 'set_beneficiary', {
    account_id: 'foo',
    bps: 1000,
  });
  await owner.call(contract, 'set_beneficiary', {
    account_id: 'bar',
    bps: 5000,
  });

  const twoValues = await owner.call(contract, 'get_beneficiaries', {});

  t.deepEqual(twoValues, {
    foo: 1000,
    bar: 5000,
  });

  await owner.call(contract, 'remove_beneficiary', {
    account_id: 'foo',
  });

  const oneValue = await owner.call(contract, 'get_beneficiaries', {});

  t.deepEqual(oneValue, {
    bar: 5000,
  });
});
