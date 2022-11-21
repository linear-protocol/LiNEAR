import { NearAccount, NEAR, Gas } from "near-workspaces-ava";
import { assertFailure, initWorkSpace, createStakingPool, setManager } from "./helper";

const workspace = initWorkSpace();

function assertValidatorAmountHelper (
  test: any,
  contract: NearAccount,
  owner: NearAccount
) {
  return async function (
    validator: NearAccount, 
    stakedAmount: string,
    unstakedAmount: string
  ) {
    // 1. make sure validator has correct balance
    test.is(
      await validator.view('get_account_staked_balance', { account_id: contract.accountId }),
      NEAR.parse(stakedAmount).toString()
    );
    test.is(
      await validator.view('get_account_unstaked_balance', { account_id: contract.accountId }),
      NEAR.parse(unstakedAmount).toString()
    );

    // 2. make sure contract validator object is synced
    const v: any = await contract.view(
      'get_validator',
      {
        validator_id: validator.accountId
      }
    );
    const staked = NEAR.from(v.staked_amount);
    const unstaked = NEAR.from(v.unstaked_amount);
    test.is(
      staked.toString(),
      NEAR.parse(stakedAmount).toString()
    );
    test.is(
      unstaked.toString(),
      NEAR.parse(unstakedAmount).toString()
    );
  }
}

async function stakeAll (signer: NearAccount, contract: NearAccount) {
  let run = true;
  while (run) {
    run = await signer.call(
      contract,
      'epoch_stake',
      {},
      {
        gas: Gas.parse('200 Tgas')
      }
    );
  }
}

workspace.test('Non-manager call drain methods', async (test, {contract, alice}) => {
    await assertFailure(
        test,
        alice.call(
            contract,
            'drain_unstake',
            {
                validator_id: 'foo'
            }
        ),
        'Only manager can perform this action'
    );

    await assertFailure(
        test,
        alice.call(
            contract,
            'drain_withdraw',
            {
                validator_id: 'foo'
            }
        ),
        'Only manager can perform this action'
    );
});

workspace.test('drain constraints', async (test, {contract, root, owner, alice, bob}) => {
  const manager = alice;
  await setManager(root, contract, owner, manager);

  const v1 = await createStakingPool(root, 'v1');
  // add validator
  await manager.call(
    contract,
    'add_validator',
    {
      validator_id: v1.accountId,
      weight: 10
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );

  // user stake
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse('50')
    }
  );

  // run stake
  await bob.call(
    contract,
    'epoch_stake',
    {},
    {
      gas: Gas.parse('200 Tgas')
    }
  );

  // 1. cannot drain unstake when weight > 0
  await assertFailure(
    test,
    manager.call(
      contract,
      'drain_unstake',
      {
        validator_id: v1.accountId
      },
      {
        gas: Gas.parse('200 Tgas')
      }
    ),
    'Validator weight must be zero for drain operation'
  );

  // set weight to 0
  await manager.call(
    contract,
    'update_weight',
    {
      validator_id: v1.accountId,
      weight: 0
    }
  );

  // fast-forward
  await owner.call(
    contract,
    'set_epoch_height',
    { epoch: 11 }
  );

  // user unstake
  await alice.call(
    contract,
    'unstake_all',
    {}
  );

  await bob.call(
    contract,
    'epoch_unstake',
    {},
    {
      gas: Gas.parse('200 Tgas')
    }
  );

  // validator now have unstaked balance > 0
  const assertValidator = assertValidatorAmountHelper(test, contract, owner);
  await assertValidator(v1, '10', '50');

  // -- 2. cannot drain unstake when pending release
  await assertFailure(
    test,
    manager.call(
      contract,
      'drain_unstake',
      {
        validator_id: v1.accountId
      },
      {
        gas: Gas.parse('200 Tgas')
      }
    ),
    'Cannot unstake from a pending release validator'
  );

  // fast-forward
  await owner.call(
    contract,
    'set_epoch_height',
    { epoch: 15 }
  );

  // -- 3. cannot drain unstake when unstaked balance > 0
  await assertFailure(
    test,
    manager.call(
      contract,
      'drain_unstake',
      {
        validator_id: v1.accountId
      },
      {
        gas: Gas.parse('200 Tgas')
      }
    ),
    'Validator unstaked amount must be zero when drain unstake'
  );
});

workspace.test('drain unstake and withdraw', async (test, {contract, root, owner, alice, bob}) => {
  const manager = alice;
  await setManager(root, contract, owner, manager);

  const v1 = await createStakingPool(root, 'v1');
  const v2 = await createStakingPool(root, 'v2');

  // add validator
  await manager.call(
    contract,
    'add_validator',
    {
      validator_id: v1.accountId,
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
      validator_id: v2.accountId,
      weight: 10
    },
    {
      gas: Gas.parse('100 Tgas')
    }
  );

  // user stake
  await alice.call(
    contract,
    'deposit_and_stake',
    {},
    {
      attachedDeposit: NEAR.parse('50')
    }
  );

  // run stake
  await stakeAll(bob, contract);

  /**
   * Steps to drain a validator
   * 1. set weight to 0
   * 2. call drain_unstake
   * 3. call drain_withdraw
   */

  await manager.call(
    contract,
    'update_weight',
    {
      validator_id: v1.accountId,
      weight: 0
    }
  );

  await manager.call(
    contract,
    'drain_unstake',
    {
      validator_id: v1.accountId
    },
    {
      gas: Gas.parse('200 Tgas')
    }
  );

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
      validator_id: v1.accountId
    },
    {
      gas: Gas.parse('200 Tgas')
    }
  );

  // make sure v1 is drained
  const assertValidator = assertValidatorAmountHelper(test, contract, owner);
  await assertValidator(v1, '0', '0');
  await assertValidator(v2, '30', '0');

  // restake and make sure funds are re-distributed
  await stakeAll(bob, contract);

  await assertValidator(v1, '0', '0');
  await assertValidator(v2, '60', '0');
});
