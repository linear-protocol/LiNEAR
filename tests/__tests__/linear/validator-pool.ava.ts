import { Gas, NEAR } from "near-units";
import { NearAccount } from "near-workspaces-ava";
import { assertFailure, initAndSetWhitelist, initWorkSpace, } from "./helper";

const workspace = initWorkSpace();

async function setManager(root: NearAccount, contract: NearAccount, owner: NearAccount) {
  const manager = await root.createAccount('linear_manager', { initialBalance: NEAR.parse("1000000").toString() });

  // set manager
  await owner.call(
    contract,
    'add_manager',
    {
      new_manager_id: manager.accountId
    }
  );

  return manager;
}

workspace.test('not manager', async (test, { contract, alice, root, owner }) => {
  await setManager(root, contract, owner);

  let errMsg = "Only manager can perform this action";
  await assertFailure(
    test,
    alice.call(
      contract,
      'add_validator',
      {
        validator_id: 'foo',
        weight: 10
      },
      {
        gas: Gas.parse('100 Tgas')
      }
    ),
    errMsg
  );

  await assertFailure(
    test,
    alice.call(
      contract,
      'add_validators',
      {
        validator_ids: ['foo'],
        weights: [10]
      }
    ),
    errMsg
  );

  await assertFailure(
    test,
    alice.call(
      contract,
      'remove_validator',
      {
        validator_id: 'foo',
      }
    ),
    errMsg
  );

  await assertFailure(
    test,
    alice.call(
      contract,
      'update_weight',
      {
        validator_id: 'foo',
        weight: 10
      }
    ),
    errMsg
  );

  await assertFailure(
    test,
    alice.call(
      contract,
      'update_base_stake_amounts',
      {
        validator_ids: ['foo'],
        amounts: [NEAR.parse("25,000")]
      }
    ),
    errMsg
  );
});

workspace.test('add validator', async (test, context) => {
  const { root, owner, contract } = context;
  const manager = await setManager(root, contract, owner);

  await manager.call(
    contract,
    'add_validator',
    {
      validator_id: 'foo',
      weight: 10
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );
  test.is(
    await contract.view('get_total_weight'),
    10
  );

  await manager.call(
    contract,
    'add_validator',
    {
      validator_id: 'bar',
      weight: 20
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );
  test.is(
    await contract.view('get_total_weight'),
    30
  );

  const validators: [any] = await contract.view(
    'get_validators',
    {
      offset: 0,
      limit: 10
    }
  );

  test.is(
    validators.filter(v => v.account_id === 'foo')[0].weight,
    10
  );
  test.is(
    validators.filter(v => v.account_id === 'bar')[0].weight,
    20
  );
});

workspace.test('bulk add a few validators', async (test, context) => {
  const { root, owner, contract } = context;
  const manager = await setManager(root, contract, owner);

  await manager.call(
    contract,
    'add_validators',
    {
      validator_ids: ['foo', 'bar'],
      weights: [10, 20]
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );

  test.is(
    await contract.view('get_total_weight'),
    30
  );

  const validators: [any] = await contract.view(
    'get_validators',
    {
      offset: 0,
      limit: 10
    }
  );

  test.is(
    validators.filter(v => v.account_id === 'foo')[0].weight,
    10
  );
  test.is(
    validators.filter(v => v.account_id === 'bar')[0].weight,
    20
  );
});

workspace.test('bulk add a lot validators', async (test, context) => {
  const { root, owner, contract } = context;
  const manager = await setManager(root, contract, owner);

  for (let i = 0; i < 2; i++) {
    const validators = Array.from({ length: 5 }, (_, j) => `validator-${i}-${j}`);
    const weights = validators.map(_ => 1);

    await manager.call(
      contract,
      'add_validators',
      {
        validator_ids: validators,
        weights
      },
      {
        gas: Gas.parse('300 Tgas')
      }
    );
  }

  test.is(
    await contract.view('get_total_weight'),
    10
  );

  // read all validators
  for (let i = 0; i < 2; i++) {
    const limit = 5;
    const offset = i * limit;

    await manager.call(
      contract,
      'get_validators',
      {
        offset,
        limit
      },
      {
        gas: Gas.parse('200 Tgas')
      }
    );
  }
});

workspace.test('whitelist', async (test, context) => {
  const { root, owner, contract } = context;

  // set a new whitelist
  const whitelist = await initAndSetWhitelist(root, contract, owner, false);

  // set whitelist account
  await root.call(
    whitelist,
    'add_whitelist',
    {
      account_id: 'foo'
    }
  );

  // try to add an validator not in whitelist
  await owner.call(
    contract,
    'add_validators',
    {
      validator_ids: ['bar'],
      weights: [1]
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );

  let validators: any[] = await contract.view(
    'get_validators',
    {
      offset: 0,
      limit: 10
    }
  );
  test.assert(validators.length === 0, 'bar should not be added');

  // try to add an validator in whitelist
  await owner.call(
    contract,
    'add_validators',
    {
      validator_ids: ['foo'],
      weights: [1]
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );

  validators = await contract.view(
    'get_validators',
    {
      offset: 0,
      limit: 10
    }
  );
  test.assert(validators.length === 1, 'foo should be added');
  test.assert(
    validators[0].account_id === 'foo'
  );
});

workspace.test('remove validator', async (test, context) => {
  const { root, owner, contract } = context;
  const manager = await setManager(root, contract, owner);

  // add foo, bar
  await manager.call(
    contract,
    'add_validator',
    {
      validator_id: 'foo',
      weight: 10
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );
  await manager.call(
    contract,
    'add_validator',
    {
      validator_id: 'bar',
      weight: 20
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );

  // remove foo
  await manager.call(
    contract,
    'remove_validator',
    {
      validator_id: 'foo'
    }
  );

  test.is(
    await contract.view('get_total_weight'),
    20
  );

  let validators: [any] = await contract.view(
    'get_validators',
    {
      offset: 0,
      limit: 10
    }
  );

  test.is(
    validators.length,
    1
  );
  test.is(
    validators[0].account_id,
    'bar'
  );

  // remove bar
  await manager.call(
    contract,
    'remove_validator',
    {
      validator_id: 'bar'
    }
  );
  test.is(
    await contract.view('get_total_weight'),
    0
  );

  validators = await manager.call(
    contract,
    'get_validators',
    {
      offset: 0,
      limit: 10
    }
  );

  test.is(
    validators.length,
    0
  );
});

workspace.test('update weight', async (test, context) => {
  const { root, owner, contract } = context;
  const manager = await setManager(root, contract, owner);

  // add foo, bar
  await manager.call(
    contract,
    'add_validator',
    {
      validator_id: 'foo',
      weight: 10
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );
  await manager.call(
    contract,
    'add_validator',
    {
      validator_id: 'bar',
      weight: 20
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );

  // update foo
  await manager.call(
    contract,
    'update_weight',
    {
      validator_id: 'foo',
      weight: 30
    }
  );
  test.is(
    await contract.view('get_total_weight'),
    50
  );

  // update bar
  await manager.call(
    contract,
    'update_weight',
    {
      validator_id: 'bar',
      weight: 5
    }
  );
  test.is(
    await contract.view('get_total_weight'),
    35
  );
});

workspace.test('update base stake amount', async (test, context) => {
  const { root, owner, contract } = context;
  const manager = await setManager(root, contract, owner);

  // add foo, bar
  await manager.call(
    contract,
    'add_validator',
    {
      validator_id: 'foo',
      weight: 10
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );
  await manager.call(
    contract,
    'add_validator',
    {
      validator_id: 'bar',
      weight: 20
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );

  // update base stake amount of foo and bar
  const amounts = [
    NEAR.parse("20000").toString(),
    NEAR.parse("50000").toString()
  ];
  await manager.call(
    contract,
    'update_base_stake_amounts',
    {
      validator_ids: [
        'foo',
        'bar'
      ],
      amounts
    }
  );

  const foo: any = await contract.view(
    'get_validator',
    {
      validator_id: 'foo'
    }
  );
  test.is(
    foo.base_stake_amount,
    amounts[0]
  );

  const bar: any = await contract.view(
    'get_validator',
    {
      validator_id: 'bar'
    }
  );
  test.is(
    bar.base_stake_amount,
    amounts[1]
  );
});
