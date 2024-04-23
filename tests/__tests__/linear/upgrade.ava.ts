import { readFileSync } from 'fs';
import { Gas, NEAR } from 'near-units';
import { NearAccount, Worker } from 'near-workspaces';
import {
  createAndDeploy,
  createStakingPool,
  epochHeightFastforward,
  epochStake,
  epochUnstake,
  epochWithdraw,
  getValidator,
  initAndSetWhitelist,
  updateBaseStakeAmounts,
} from './helper';
import anyTest, { TestFn } from 'ava';

const test = anyTest as TestFn<WorkSpace>;

interface WorkSpace {
  worker: Worker;
  root: NearAccount;
  contract: NearAccount;
  owner: NearAccount;
  manager: NearAccount;
  alice: NearAccount;
  bob: NearAccount;
  carol: NearAccount;
}

async function deployLinearAtVersion(
  root: NearAccount,
  owner_id: string,
  version: string,
) {
  return createAndDeploy(
    root,
    'linear',
    `compiled-contracts/linear_${version}.wasm`,
    {
      methodName: 'new',
      args: {
        owner_id,
      },
    },
  );
}

async function upgrade(contract: NearAccount, owner: NearAccount) {
  await owner.call(
    contract,
    'upgrade',
    readFileSync('compiled-contracts/linear.wasm'),
    {
      gas: Gas.parse('300 Tgas'),
    },
  );
}

async function stakeAll(signer: NearAccount, contract: NearAccount) {
  let run = true;
  while (run) {
    run = await epochStake(signer, contract);
  }
}

async function unstakeAll(signer: NearAccount, contract: NearAccount) {
  let run = true;
  while (run) {
    run = await epochUnstake(signer, contract);
  }
}

async function withdrawAll(signer: NearAccount, contract: NearAccount) {
  const validators: any[] = await contract.view('get_validators', {
    offset: 0,
    limit: 100,
  });
  for (const validator of validators) {
    await epochWithdraw(contract, signer, validator.account_id);
  }
}

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

async function initWorkSpace(version: string): Promise<WorkSpace> {
  const worker = await Worker.init({
    network: 'sandbox',
    rm: true,
  });

  const root = worker.rootAccount;

  // deposit 1M $NEAR for each account
  const owner = await root.createSubAccount('linear_owner', {
    initialBalance: NEAR.parse('1000000').toString(),
  });
  const alice = await root.createSubAccount('alice', {
    initialBalance: NEAR.parse('1000000').toString(),
  });
  const bob = await root.createSubAccount('bob', {
    initialBalance: NEAR.parse('1000000').toString(),
  });
  const carol = await root.createSubAccount('carol', {
    initialBalance: NEAR.parse('1000000').toString(),
  });

  const contract = await deployLinearAtVersion(root, owner.accountId, version);

  await initAndSetWhitelist(root, contract, owner, true);
  const manager = await setManager(root, contract, owner);

  return { worker, root, contract, owner, manager, alice, bob, carol };
}

const BASE_VERSION = 'v1_5_1'; // change this to the version that you want to upgrade from

test.beforeEach(async (t) => {
  t.context = await initWorkSpace(BASE_VERSION);
});

test.afterEach(async (t) => {
  await t.context.worker.tearDown();
});

// The upgrade() test has run successfully in sandbox by migrating the states of 50 validators.
// Skip the test in CI because upgrade varies between versions and is almost a one-time effort.
// Keep this test case to make it easier to be reused in future upgrade.
test.skip('upgrade contract from v1.2.0 to v1.3.0 on testnet', async (t) => {
  const { root, contract, owner, manager, alice, bob } = t.context;

  const groups = 10;
  const limit = 5;

  // set up validators
  for (let i = 0; i < groups; i++) {
    const names = Array.from(
      { length: limit },
      (_, j) => `validator-${i}-${j}`,
    );
    const weights = names.map((_) => 1);
    const validators = await Promise.all(
      names.map((name) => createStakingPool(root, name)),
    );

    await manager.call(
      contract,
      'add_validators',
      {
        validator_ids: validators.map((v) => v.accountId),
        weights,
      },
      {
        gas: Gas.parse('300 Tgas'),
      },
    );
  }

  t.is(await contract.view('get_total_weight'), groups * limit);

  // user stake
  const staked = 5000;
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse((staked - 10).toFixed(0)),
    },
  );

  // run epoch stake
  await stakeAll(bob, contract);

  // read all validators
  for (let i = 0; i < groups; i++) {
    const offset = i * limit;

    const validators: any = await contract.view('get_validators', {
      offset,
      limit,
    });
    for (const v of validators) {
      t.is(
        v.staked_amount,
        NEAR.parse((staked / groups / limit).toFixed(0)).toString(),
      );
      t.is(v.base_stake_amount, undefined);
    }
  }

  // upgrade linear contract to the latest
  await upgrade(contract, owner);

  // read all validators
  for (let i = 0; i < groups; i++) {
    const offset = i * limit;

    const validators: any = await contract.view('get_validators', {
      offset,
      limit,
    });
    for (const v of validators) {
      t.is(
        v.staked_amount,
        NEAR.parse((staked / groups / limit).toFixed(0)).toString(),
      );
      t.is(v.base_stake_amount, '0');
    }
  }

  t.is(await contract.view('get_total_weight'), groups * limit);

  // update base stake amount after upgrade

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

// test drain unstake and withdraw
test.skip('upgrade from v1.3.3 to v1.4.0', async (t) => {
  const { root, contract, owner, manager, alice } = t.context;

  // add some validators
  const names = Array.from({ length: 5 }, (_, i) => `validator-${i}`);
  const weights = names.map((_) => 1);
  const validators = await Promise.all(
    names.map((name) => createStakingPool(root, name)),
  );

  await manager.call(
    contract,
    'add_validators',
    {
      validator_ids: validators.map((v) => v.accountId),
      weights,
    },
    {
      gas: Gas.parse('300 Tgas'),
    },
  );

  // user stake
  const staked = 5000;
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse((staked - 10).toFixed(0)),
    },
  );

  // run epoch stake
  await stakeAll(manager, contract);

  // upgrade linear contract to the latest
  await upgrade(contract, owner);

  // read validators to verify upgrade
  for (const validator of validators) {
    const v = await getValidator(contract, validator.accountId);
    t.assert(v.draining === false);
  }

  // try to drain one of the validators
  const targetValidator = validators[0];

  // set weight to 0
  await manager.call(contract, 'update_weight', {
    validator_id: targetValidator.accountId,
    weight: 0,
  });

  await manager.call(
    contract,
    'drain_unstake',
    {
      validator_id: targetValidator.accountId,
    },
    {
      gas: Gas.parse('275 Tgas'),
    },
  );

  t.assert((await getValidator(contract, targetValidator.accountId)).draining);

  // fast-forward
  await owner.call(contract, 'set_epoch_height', { epoch: 14 });

  await manager.call(
    contract,
    'drain_withdraw',
    {
      validator_id: targetValidator.accountId,
    },
    {
      gas: Gas.parse('200 Tgas'),
    },
  );

  t.assert(!(await getValidator(contract, targetValidator.accountId)).draining);
});

// test validator execution status
test.skip('upgrade from v1.4.4 to v1.5.0', async (t) => {
  const { root, contract, owner, manager, alice } = t.context;

  // add some validators
  const names = Array.from({ length: 5 }, (_, i) => `validator-${i}`);
  const weights = names.map((_) => 1);
  const validators = await Promise.all(
    names.map((name) => createStakingPool(root, name)),
  );

  await manager.call(
    contract,
    'add_validators',
    {
      validator_ids: validators.map((v) => v.accountId),
      weights,
    },
    {
      gas: Gas.parse('300 Tgas'),
    },
  );

  // user stake
  const staked = 500;
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse((staked - 10).toFixed(0)),
    },
  );

  // run epoch stake
  await stakeAll(manager, contract);

  // upgrade linear contract to the latest
  await upgrade(contract, owner);

  // read validators to verify upgrade
  for (const validator of validators) {
    const v = await getValidator(contract, validator.accountId);
    t.true(!v.executing);
  }

  // fast-forward
  await owner.call(contract, 'set_epoch_height', { epoch: 11 });

  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse(staked.toFixed(0)),
    },
  );

  async function sleep(ms: number) {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  async function delayedEpochStake(ms: number) {
    await sleep(ms);
    await epochStake(alice, contract);
  }

  let executed = false;
  async function watch() {
    for (let i = 0; i < 10; i++) {
      await sleep(500);
      const info = await getValidator(contract, validators[0].accountId);
      if (info.executing) {
        executed = true;
      }
    }
  }

  // stake to validators, and watch execution status
  await Promise.all([delayedEpochStake(1000), watch()]);

  // once be executing
  t.true(executed);
});

// regression test after upgrade
test.skip('upgrade from v1.5.1 to v1.6.0', async (t) => {
  const { root, contract, owner, manager, alice } = t.context;

  // add some validators
  const names = Array.from({ length: 5 }, (_, i) => `validator-${i}`);
  const weights = names.map((_) => 1);
  const validators = await Promise.all(
    names.map((name) => createStakingPool(root, name)),
  );

  await manager.call(
    contract,
    'add_validators',
    {
      validator_ids: validators.map((v) => v.accountId),
      weights,
    },
    {
      gas: Gas.parse('300 Tgas'),
    },
  );

  // user stakes
  const stakeAmount = NEAR.parse('4900');
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: stakeAmount,
    },
  );

  // run epoch stake
  await stakeAll(manager, contract);

  // wait 1 epoch
  await epochHeightFastforward(contract, alice, 1);

  // upgrade linear contract to the latest
  await upgrade(contract, owner);

  // unstake
  const unstakeAmount = NEAR.parse('500');
  await alice.call(contract, 'unstake', { amount: unstakeAmount.toString() });

  // run epoch unstake
  await unstakeAll(manager, contract);

  // wait 4 epoches
  await epochHeightFastforward(contract, alice);
  await withdrawAll(manager, contract);

  // withdraw all after 4 epoches
  await alice.call(contract, 'withdraw_all', {});

  t.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    stakeAmount.sub(unstakeAmount).toString(),
  );
  t.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    '0',
  );

  // unstake all
  await alice.call(contract, 'unstake_all', {});

  // run epoch unstake
  await unstakeAll(manager, contract);

  // wait 4 epoches
  await epochHeightFastforward(contract, alice);
  await withdrawAll(manager, contract);

  // withdraw all after 4 epoches
  const withdrawAmount = NEAR.parse('1');
  await alice.call(contract, 'withdraw', { amount: withdrawAmount.toString() });

  t.is(
    await contract.view('get_account_staked_balance', { account_id: alice }),
    '0',
  );
  t.is(
    await contract.view('get_account_unstaked_balance', { account_id: alice }),
    stakeAmount.sub(unstakeAmount).sub(withdrawAmount).toString(),
  );
});
