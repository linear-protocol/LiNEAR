import { Gas, NEAR } from "near-units";
import { BN, NearAccount } from "near-workspaces-ava";
import { assertFailure, getValidator, initAndSetWhitelist, initWorkSpace, updateBaseStakeAmounts, } from "./helper";

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
    updateBaseStakeAmounts(
      contract,
      alice,
      ['foo'],
      [NEAR.parse("25,000")]
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

workspace.test('update weights', async (test, context) => {
  const { root, owner, contract } = context;
  const manager = await setManager(root, contract, owner);

  // add foo, bar
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

  // update foo
  await manager.call(
    contract,
    'update_weights',
    {
      validator_ids: ['foo', 'bar'],
      weights: [30, 5]
    }
  );
  test.is(
    await contract.view('get_total_weight'),
    35
  );
});

workspace.test('max update weights', async (test, context) => {
  const { root, owner, contract } = context;
  const manager = await setManager(root, contract, owner);

  const validator_ids: string[] = [];
  let weights: number[] = [];

  let totalWeight = 0;
  const total = 30;
  for (let i = 0; i < total; i++) {
    totalWeight += i;

    const id = i.toFixed(0) + ".test.near";
    const weight = i;

    validator_ids.push(id);
    weights.push(weight);
  }

  const delta = 5;
  for (let i = 0; i < total / delta; i++) {
    await manager.call(
      contract,
      'add_validators',
      {
        validator_ids: validator_ids.slice(i * delta, i * delta + delta),
        weights: weights.slice(i * delta, i * delta + delta),
      },
      {
        gas: Gas.parse('300 Tgas')
      }
    );
  }

  weights = weights.map(x => x + 1);
  // update foo
  const result = await manager.call_raw(
    contract,
    'update_weights',
    {
      validator_ids,
      weights,
    },
    {
      gas: Gas.parse('300 Tgas')
    }
  );
  console.log("gas_burnt", result.outcome.reduce((pre, o) => {
    return pre.add(new BN(o.gas_burnt));
  } , new BN(0)).toString(10));
  test.is(
    await contract.view('get_total_weight'),
    totalWeight + total
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
    NEAR.parse("20000"),
    NEAR.parse("50000")
  ];
  await updateBaseStakeAmounts(
    contract,
    manager,
    [
      'foo',
      'bar'
    ],
    amounts
  );

  const foo = await getValidator(contract, 'foo');
  test.is(
    foo.base_stake_amount,
    amounts[0].toString()
  );

  const bar = await getValidator(contract, 'bar');
  test.is(
    bar.base_stake_amount,
    amounts[1].toString()
  );
});
