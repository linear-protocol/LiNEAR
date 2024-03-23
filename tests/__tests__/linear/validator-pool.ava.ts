import { Gas, NEAR } from 'near-units';
import { NearAccount } from 'near-workspaces';
import {
  assertFailure,
  getValidator,
  initAndSetWhitelist,
  initWorkspace, test,
  updateBaseStakeAmounts,
} from './helper';

async function setManager(
  root: NearAccount,
  contract: NearAccount,
  owner: NearAccount,
) {
  const manager = await root.createSubAccount('linear_manager', {
    initialBalance: NEAR.parse('1000000').toString(),
  });

  // set manager
  await owner.call(contract, 'add_manager', {
    new_manager_id: manager.accountId,
  });

  return manager;
}

test.beforeEach(async (t) => {
  t.context = await initWorkspace();
});

test.afterEach(async (t) => {
  await t.context.worker.tearDown();
});

test(
  'not manager',
  async (t) => {
    const { contract, alice, root, owner } = t.context;
    await setManager(root, contract, owner);

    let errMsg = 'Only manager can perform this action';
    await assertFailure(
      t,
      alice.call(
        contract,
        'add_validator',
        {
          validator_id: 'foo',
          weight: 10,
        },
        {
          gas: Gas.parse('100 Tgas'),
        },
      ),
      errMsg,
    );

    await assertFailure(
      t,
      alice.call(contract, 'add_validators', {
        validator_ids: ['foo'],
        weights: [10],
      }),
      errMsg,
    );

    await assertFailure(
      t,
      alice.call(contract, 'remove_validator', {
        validator_id: 'foo',
      }),
      errMsg,
    );

    await assertFailure(
      t,
      alice.call(contract, 'update_weight', {
        validator_id: 'foo',
        weight: 10,
      }),
      errMsg,
    );

    await assertFailure(
      t,
      updateBaseStakeAmounts(contract, alice, ['foo'], [NEAR.parse('25,000')]),
      errMsg,
    );
  },
);

test('add validator', async (t) => {
  const { root, owner, contract } = t.context;
  const manager = await setManager(root, contract, owner);

  await manager.call(
    contract,
    'add_validator',
    {
      validator_id: 'foo',
      weight: 10,
    },
    {
      gas: Gas.parse('100 Tgas'),
    },
  );
  t.is(await contract.view('get_total_weight'), 10);

  await manager.call(
    contract,
    'add_validator',
    {
      validator_id: 'bar',
      weight: 20,
    },
    {
      gas: Gas.parse('100 Tgas'),
    },
  );
  t.is(await contract.view('get_total_weight'), 30);

  const validators: [any] = await contract.view('get_validators', {
    offset: 0,
    limit: 10,
  });

  t.is(validators.filter((v) => v.account_id === 'foo')[0].weight, 10);
  t.is(validators.filter((v) => v.account_id === 'bar')[0].weight, 20);
});

test('bulk add a few validators', async (t) => {
  const { root, owner, contract } = t.context;
  const manager = await setManager(root, contract, owner);

  await manager.call(
    contract,
    'add_validators',
    {
      validator_ids: ['foo', 'bar'],
      weights: [10, 20],
    },
    {
      gas: Gas.parse('100 Tgas'),
    },
  );

  t.is(await contract.view('get_total_weight'), 30);

  const validators: [any] = await contract.view('get_validators', {
    offset: 0,
    limit: 10,
  });

  t.is(validators.filter((v) => v.account_id === 'foo')[0].weight, 10);
  t.is(validators.filter((v) => v.account_id === 'bar')[0].weight, 20);
});

test('bulk add a lot validators', async (t) => {
  const { root, owner, contract } = t.context;
  const manager = await setManager(root, contract, owner);

  for (let i = 0; i < 2; i++) {
    const validators = Array.from(
      { length: 5 },
      (_, j) => `validator-${i}-${j}`,
    );
    const weights = validators.map((_) => 1);

    await manager.call(
      contract,
      'add_validators',
      {
        validator_ids: validators,
        weights,
      },
      {
        gas: Gas.parse('300 Tgas'),
      },
    );
  }

  t.is(await contract.view('get_total_weight'), 10);

  // read all validators
  for (let i = 0; i < 2; i++) {
    const limit = 5;
    const offset = i * limit;

    await manager.call(
      contract,
      'get_validators',
      {
        offset,
        limit,
      },
      {
        gas: Gas.parse('200 Tgas'),
      },
    );
  }
});

test('whitelist', async (t) => {
  const { root, owner, contract } = t.context;

  // set a new whitelist
  const whitelist = await initAndSetWhitelist(root, contract, owner, false);

  // set whitelist account
  await root.call(whitelist, 'add_whitelist', {
    account_id: 'foo',
  });

  // try to add an validator not in whitelist
  await owner.call(
    contract,
    'add_validators',
    {
      validator_ids: ['bar'],
      weights: [1],
    },
    {
      gas: Gas.parse('100 Tgas'),
    },
  );

  let validators: any[] = await contract.view('get_validators', {
    offset: 0,
    limit: 10,
  });
  t.assert(validators.length === 0, 'bar should not be added');

  // try to add an validator in whitelist
  await owner.call(
    contract,
    'add_validators',
    {
      validator_ids: ['foo'],
      weights: [1],
    },
    {
      gas: Gas.parse('100 Tgas'),
    },
  );

  validators = await contract.view('get_validators', {
    offset: 0,
    limit: 10,
  });
  t.assert(validators.length === 1, 'foo should be added');
  t.assert(validators[0].account_id === 'foo');
});

test('remove validator', async (t) => {
  const { root, owner, contract } = t.context;
  const manager = await setManager(root, contract, owner);

  // add foo, bar
  await manager.call(
    contract,
    'add_validator',
    {
      validator_id: 'foo',
      weight: 10,
    },
    {
      gas: Gas.parse('100 Tgas'),
    },
  );
  await manager.call(
    contract,
    'add_validator',
    {
      validator_id: 'bar',
      weight: 20,
    },
    {
      gas: Gas.parse('100 Tgas'),
    },
  );

  // remove foo
  await manager.call(contract, 'remove_validator', {
    validator_id: 'foo',
  });

  t.is(await contract.view('get_total_weight'), 20);

  let validators = await contract.view<any[]>('get_validators', {
    offset: 0,
    limit: 10,
  });

  t.is(validators.length, 1);
  t.is(validators[0].account_id, 'bar');

  // remove bar
  await manager.call(contract, 'remove_validator', {
    validator_id: 'bar',
  });
  t.is(await contract.view('get_total_weight'), 0);

  validators = await manager.call<any[]>(contract, 'get_validators', {
    offset: 0,
    limit: 10,
  });

  t.is(validators.length, 0);
});

test('update weight', async (t) => {
  const { root, owner, contract } = t.context;
  const manager = await setManager(root, contract, owner);

  // add foo, bar
  await manager.call(
    contract,
    'add_validator',
    {
      validator_id: 'foo',
      weight: 10,
    },
    {
      gas: Gas.parse('100 Tgas'),
    },
  );
  await manager.call(
    contract,
    'add_validator',
    {
      validator_id: 'bar',
      weight: 20,
    },
    {
      gas: Gas.parse('100 Tgas'),
    },
  );

  // update foo
  await manager.call(contract, 'update_weight', {
    validator_id: 'foo',
    weight: 30,
  });
  t.is(await contract.view('get_total_weight'), 50);

  // update bar
  await manager.call(contract, 'update_weight', {
    validator_id: 'bar',
    weight: 5,
  });
  t.is(await contract.view('get_total_weight'), 35);
});

test('update weights', async (t) => {
  const { root, owner, contract } = t.context;
  const manager = await setManager(root, contract, owner);

  // add foo, bar
  await manager.call(
    contract,
    'add_validators',
    {
      validator_ids: ['foo', 'bar'],
      weights: [10, 20],
    },
    {
      gas: Gas.parse('100 Tgas'),
    },
  );

  // update foo
  await manager.call(contract, 'update_weights', {
    validator_ids: ['foo', 'bar'],
    weights: [30, 5],
  });
  t.is(await contract.view('get_total_weight'), 35);
});

test('update base stake amount', async (t) => {
  const { root, owner, contract } = t.context;
  const manager = await setManager(root, contract, owner);

  // add foo, bar
  await manager.call(
    contract,
    'add_validator',
    {
      validator_id: 'foo',
      weight: 10,
    },
    {
      gas: Gas.parse('100 Tgas'),
    },
  );
  await manager.call(
    contract,
    'add_validator',
    {
      validator_id: 'bar',
      weight: 20,
    },
    {
      gas: Gas.parse('100 Tgas'),
    },
  );

  // update base stake amount of foo and bar
  const amounts = [NEAR.parse('20000'), NEAR.parse('50000')];
  await updateBaseStakeAmounts(contract, manager, ['foo', 'bar'], amounts);

  const foo = await getValidator(contract, 'foo');
  t.is(foo.base_stake_amount, amounts[0].toString());

  const bar = await getValidator(contract, 'bar');
  t.is(bar.base_stake_amount, amounts[1].toString());
});
