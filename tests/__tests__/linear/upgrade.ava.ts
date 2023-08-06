import { readFileSync } from "fs";
import { Gas, NEAR } from "near-units";
import { NearAccount, Workspace } from "near-workspaces-ava";
import { createStakingPool, getValidator, initAndSetWhitelist, skip, updateBaseStakeAmounts, } from "./helper";

async function deployLinearAtVersion(
  root: NearAccount,
  owner_id: string,
  version: string,
) {
  return root.createAndDeploy(
    'linear',
    `compiled-contracts/linear_${version}.wasm`,
    {
      method: 'new',
      args: {
        owner_id,
      }
    }
  )
}

async function upgrade(contract: NearAccount, owner: NearAccount) {
  await owner.call(
    contract,
    "upgrade",
    readFileSync("compiled-contracts/linear.wasm"),
    {
      gas: Gas.parse("300 Tgas"),
    }
  );
}

async function stakeAll (signer: NearAccount, contract: NearAccount) {
  let run = true;
  while (run) {
    run = await signer.call(
      contract,
      'epoch_stake',
      {},
      {
        gas: Gas.parse('300 Tgas')
      }
    );
  }
}

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

function initWorkSpace(version: string) {
  return Workspace.init(async ({ root }) => {
    // deposit 1M $NEAR for each account
    const owner = await root.createAccount('linear_owner', { initialBalance: NEAR.parse("1000000").toString() });
    const alice = await root.createAccount('alice', { initialBalance: NEAR.parse("1000000").toString() });
    const bob = await root.createAccount('bob', { initialBalance: NEAR.parse("1000000").toString() });
    const carol = await root.createAccount('carol', { initialBalance: NEAR.parse("1000000").toString() });

    const contract = await deployLinearAtVersion(root, owner.accountId, version);

    await initAndSetWhitelist(root, contract, owner, true);
    const manager = await setManager(root, contract, owner);

    return { contract, owner, manager, alice, bob, carol, };
  });
}

const baseVersion = 'v1_4_4';  // change this to the version that you want to upgrade from
const workspace = initWorkSpace(baseVersion);

// The upgrade() test has run successfully in sandbox by migrating the states of 50 validators.
// Skip the test in CI because upgrade varies between versions and is almost a one-time effort.
// Keep this test case to make it easier to be reused in future upgrade.
skip('upgrade contract from v1.2.0 to v1.3.0 on testnet', async (test, context) => {
  const { root, contract, owner, manager, alice, bob } = context;

  const groups = 10;
  const limit = 5;

  // set up validators
  for (let i = 0; i < groups; i++) {
    const names = Array.from({ length: limit }, (_, j) => `validator-${i}-${j}`);
    const weights = names.map(_ => 1);
    const validators = await Promise.all(names.map(name => createStakingPool(root, name)));

    await manager.call(
      contract,
      'add_validators',
      {
        validator_ids: validators.map(v => v.accountId),
        weights
      },
      {
        gas: Gas.parse('300 Tgas')
      }
    );
  }

  test.is(
    await contract.view('get_total_weight'),
    groups * limit
  );

  // user stake
  const staked = 5000;
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse((staked-10).toFixed(0))
    }
  );

  // run epoch stake
  await stakeAll(bob, contract);


  // read all validators
  for (let i = 0; i < groups; i++) {
    const offset = i * limit;

    const validators: any = await contract.view(
      'get_validators',
      {
        offset,
        limit
      }
    );
    for (const v of validators) {
      test.is(
        v.staked_amount,
        NEAR.parse((staked / groups / limit).toFixed(0)).toString()
      );
      test.is(
        v.base_stake_amount,
        undefined
      );
    }
  }

  // upgrade linear contract to the latest
  await upgrade(contract, owner);

  // read all validators
  for (let i = 0; i < groups; i++) {
    const offset = i * limit;

    const validators: any = await contract.view(
      'get_validators',
      {
        offset,
        limit
      }
    );
    for (const v of validators) {
      test.is(
        v.staked_amount,
        NEAR.parse((staked / groups / limit).toFixed(0)).toString()
      );
      test.is(
        v.base_stake_amount,
        '0'
      );
    }
  }

  test.is(
    await contract.view('get_total_weight'),
    groups * limit
  );


  // update base stake amount after upgrade

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

skip('upgrade from v1.3.3 to v1.4.0', async (test, context) => {
  const { root, contract, owner, manager, alice } = context;

  // add some validators
  const names = Array.from({ length: 5 }, (_, i) => `validator-${i}`);
  const weights = names.map(_ => 1);
  const validators = await Promise.all(names.map(name => createStakingPool(root, name)));

  await manager.call(
    contract,
    'add_validators',
    {
      validator_ids: validators.map(v => v.accountId),
      weights
    },
    {
      gas: Gas.parse('300 Tgas')
    }
  );

  // user stake
  const staked = 5000;
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse((staked-10).toFixed(0))
    }
  );

  // run epoch stake
  await stakeAll(manager, contract);

  // upgrade linear contract to the latest
  await upgrade(contract, owner);

  // read validators to verify upgrade
  for (const validator of validators) {
    const v = await getValidator(contract, validator.accountId);
    test.assert(v.draining === false);
  }

  // try to drain one of the validators
  const targetValidator = validators[0];

  // set weight to 0
  await manager.call(
    contract,
    'update_weight',
    {
      validator_id: targetValidator.accountId,
      weight: 0
    }
  );

  await manager.call(
    contract,
    'drain_unstake',
    {
      validator_id: targetValidator.accountId
    },
    {
      gas: Gas.parse('200 Tgas')
    }
  );

  test.assert((await getValidator(contract, targetValidator.accountId)).draining);

  // fast-forward
  await owner.call(
    contract,
    'set_epoch_height',
    { epoch: 14 }
  );

  await manager.call(
    contract,
    'drain_withdraw',
    {
      validator_id: targetValidator.accountId
    },
    {
      gas: Gas.parse('200 Tgas')
    }
  );

  test.assert(!(await getValidator(contract, targetValidator.accountId)).draining);
});

skip('upgrade from v1.4.4 to v1.5.0', async (test, context) => {
  const { root, contract, owner, manager, alice } = context;

  // add some validators
  const names = Array.from({ length: 5 }, (_, i) => `validator-${i}`);
  const weights = names.map(_ => 1);
  const validators = await Promise.all(names.map(name => createStakingPool(root, name)));

  await manager.call(
    contract,
    'add_validators',
    {
      validator_ids: validators.map(v => v.accountId),
      weights
    },
    {
      gas: Gas.parse('300 Tgas')
    }
  );

  // user stake
  const staked = 500;
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse((staked-10).toFixed(0))
    }
  );

  // run epoch stake
  await stakeAll(manager, contract);

  // upgrade linear contract to the latest
  await upgrade(contract, owner);

  // read validators to verify upgrade
  for (const validator of validators) {
    const v = await getValidator(contract, validator.accountId);
    test.true(!v.executing);
  }

  // fast-forward
  await owner.call(
    contract,
    'set_epoch_height',
    { epoch: 11 }
  );

  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse((staked).toFixed(0))
    }
  );

  async function sleep(ms: number) {
    return new Promise( resolve => setTimeout(resolve, ms) );
  }

  async function delayedEpochStake(ms: number) {
    await sleep(ms);
    await alice.call(
      contract,
      'epoch_stake',
      {},
      {
        gas: Gas.parse('300 Tgas')
      }
    );
  }

  let executed = false;
  async function watch() {
    for (let i = 0; i < 10; i++) {
      await sleep(500);
      const info = await getValidator(contract, validators[0].accountId);
      console.log('info', info);
      if (info.executing) {
        executed = true;
      }
    }
  }

  // stake to validators, and watch execution status
  await Promise.all([
    delayedEpochStake(1000),
    watch()
  ]);

  // once be executing
  test.true(executed);
});
