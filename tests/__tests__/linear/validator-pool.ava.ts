import { Gas, NEAR } from "near-units";
import { NearAccount } from "near-workspaces-ava";
import { assertFailure, initWorkSpace } from "./helper";

const workspace = initWorkSpace();

async function setOperator(root: NearAccount, contract: NearAccount, owner: NearAccount) {
  const operator = await root.createAccount('linear_operator', { initialBalance: NEAR.parse("1000000").toString() });

  // set operator
  await owner.call(
    contract,
    'set_operator',
    {
      new_operator_id: operator.accountId
    }
  );

  return operator;
}

workspace.test('not operator', async (test, { contract, alice, root, owner }) => {
  await setOperator(root, contract, owner);

  let errMsg = "Only operator can perform this action";
  await assertFailure(
    test,
    alice.call(
      contract,
      'add_validator',
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
      'get_validators',
      {
        offset: 0,
        limit: 1
      }
    ),
    errMsg
  );
});

workspace.test('add validator', async (test, context) => {
  const { root, owner, contract } = context;
  const operator = await setOperator(root, contract, owner);

  await operator.call(
    contract,
    'add_validator',
    {
      validator_id: 'foo',
      weight: 10
    }
  );
  test.is(
    await contract.view('get_total_weight'),
    10
  );

  await operator.call(
    contract,
    'add_validator',
    {
      validator_id: 'bar',
      weight: 20
    }
  );
  test.is(
    await contract.view('get_total_weight'),
    30
  );

  const validators: [any] = await operator.call(
    contract,
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
  const operator = await setOperator(root, contract, owner);

  await operator.call(
    contract,
    'add_validators',
    {
      validator_ids: ['foo', 'bar'],
      weights: [10, 20]
    }
  );

  test.is(
    await contract.view('get_total_weight'),
    30
  );

  const validators: [any] = await operator.call(
    contract,
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
  const operator = await setOperator(root, contract, owner);

  for (let i = 0; i < 2; i++) {
    const validators = Array.from({ length: 50 }, (_, j) => `validator-${i}-${j}`);
    const weights = validators.map(_ => 1);

    await operator.call(
      contract,
      'add_validators',
      {
        validator_ids: validators,
        weights
      },
      {
        gas: Gas.parse('200 Tgas')
      }
    );
  }

  test.is(
    await contract.view('get_total_weight'),
    100
  );

  // read all validators
  for (let i = 0; i < 5; i++) {
    const limit = 20;
    const offset = i * limit;

    await operator.call(
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

workspace.test('remove validator', async (test, context) => {
  const { root, owner, contract } = context;
  const operator = await setOperator(root, contract, owner);

  // add foo, bar
  await operator.call(
    contract,
    'add_validator',
    {
      validator_id: 'foo',
      weight: 10
    }
  );
  await operator.call(
    contract,
    'add_validator',
    {
      validator_id: 'bar',
      weight: 20
    }
  );

  // remove foo
  await operator.call(
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
  let validators: [any] = await operator.call(
    contract,
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
  await operator.call(
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

  validators = await operator.call(
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
  const operator = await setOperator(root, contract, owner);

  // add foo, bar
  await operator.call(
    contract,
    'add_validator',
    {
      validator_id: 'foo',
      weight: 10
    }
  );
  await operator.call(
    contract,
    'add_validator',
    {
      validator_id: 'bar',
      weight: 20
    }
  );

  // update foo
  await operator.call(
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
  await operator.call(
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
