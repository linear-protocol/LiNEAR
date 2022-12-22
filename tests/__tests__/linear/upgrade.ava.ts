import { readFileSync } from "fs";
import { Gas, NEAR } from "near-units";
import { NearAccount, Workspace } from "near-workspaces-ava";
import { createStakingPool, initAndSetWhitelist, skip, updateBaseStakeAmounts, } from "./helper";

async function deployLinearV1_2_0(
  root: NearAccount,
  owner_id: string,
  contractId = 'linear',
) {
  return root.createAndDeploy(
    contractId,
    'compiled-contracts/linear_v1_2_0.wasm',
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
      gas: Gas.parse("200 Tgas"),
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

function initWorkSpace() {
  return Workspace.init(async ({ root }) => {
    // deposit 1M $NEAR for each account
    const owner = await root.createAccount('linear_owner', { initialBalance: NEAR.parse("1000000").toString() });
    const alice = await root.createAccount('alice', { initialBalance: NEAR.parse("1000000").toString() });
    const bob = await root.createAccount('bob', { initialBalance: NEAR.parse("1000000").toString() });
    const carol = await root.createAccount('carol', { initialBalance: NEAR.parse("1000000").toString() });

    const contract = await deployLinearV1_2_0(root, owner.accountId);

    await initAndSetWhitelist(root, contract, owner, true);

    return { contract, owner, alice, bob, carol, };
  });
}

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

// The upgrade() test has run successfully in sandbox by migrating the states of 50 validators.
// Skip the test in CI because upgrade varies between versions and is almost a one-time effort.
// Keep this test case to make it easier to be reused in future upgrade.
skip('upgrade contract from v1.2.0 to v1.3.0 on testnet', async (test, context) => {
  const { root, contract, owner, alice, bob } = context;
  const manager = await setManager(root, contract, owner);

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

  const foo: any = await contract.view(
    'get_validator',
    {
      validator_id: 'foo'
    }
  );
  test.is(
    foo.base_stake_amount,
    amounts[0].toString()
  );

  const bar: any = await contract.view(
    'get_validator',
    {
      validator_id: 'bar'
    }
  );
  test.is(
    bar.base_stake_amount,
    amounts[1].toString()
  );
});
