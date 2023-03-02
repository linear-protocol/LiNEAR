import { Workspace, NEAR, NearAccount, BN } from "near-workspaces-ava";

export const ONE_YOCTO = '1';
export const NUM_EPOCHS_TO_UNLOCK = 4;

interface RewardFee {
  numerator: number,
  denominator: number
}

export function initWorkSpace() {
  return Workspace.init(async ({ root }) => {
    // deposit 1M $NEAR for each account
    const owner = await root.createAccount('linear_owner', { initialBalance: NEAR.parse("1000000").toString() });
    const alice = await root.createAccount('alice', { initialBalance: NEAR.parse("1000000").toString() });
    const bob = await root.createAccount('bob', { initialBalance: NEAR.parse("1000000").toString() });
    const carol = await root.createAccount('carol', { initialBalance: NEAR.parse("1000000").toString() });

    const contract = await deployLinear(root, owner.accountId);

    await initAndSetWhitelist(root, contract, owner, true);

    return { contract, owner, alice, bob, carol, };
  });
}

export async function deployLinear(
  root: NearAccount,
  owner_id: string,
  contractId = 'linear',
  reward_fee?: RewardFee,
) {
  if (!reward_fee) {
    reward_fee = {
      numerator: 1,
      denominator: 100
    };
  }

  return root.createAndDeploy(
    contractId,
    'compiled-contracts/linear.wasm',
    {
      method: 'new',
      args: {
        owner_id,
        reward_fee,
      }
    }
  )
}

export async function createStakingPool (root: NearAccount, id: string) {
  return root.createAndDeploy(
    id,
    'compiled-contracts/mock_staking_pool.wasm',
    {
      method: 'new',
      args: {}
    }
  );
}

let whitelistCount = 1;

export async function initAndSetWhitelist(
  root: NearAccount, 
  contract: NearAccount,
  owner: NearAccount,
  allowAll = true
) {
  const whitelist = await root.createAndDeploy(
    `whitelist${whitelistCount++}`,
    'compiled-contracts/mock_whitelist.wasm',
    {
      method: 'new',
    }
  );

  if (allowAll) {
    await root.call(
      whitelist,
      'allow_all',
      {}
    );
  }

  await owner.call(
    contract,
    'set_whitelist_contract_id',
    {
      account_id: whitelist.accountId
    }
  );

  return whitelist;
}

function parseError(e: any): string {
  let status: any = e && e.parse
  ? e.parse().result.status
  : JSON.parse(e.message);
  return status.Failure.ActionError.kind.FunctionCallError.ExecutionError;
}

export async function assertFailure(
  test: any,
  action: Promise<unknown>,
  errorMessage?: string
) {
  let failed = false;

  try {
    await action;
  } catch (e) {
    if (errorMessage) {
      let msg: string = parseError(e);
      test.truthy(
        msg.includes(errorMessage),
        `Bad error message. expect: "${errorMessage}", actual: "${msg}"`
      );
    }
    failed = true;
  }

  test.is(
    failed,
    true,
    "Action didn't fail"
  );
}

export async function callWithMetrics(
    account: NearAccount,
    contractId: NearAccount | string,
    methodName: string,
    args: Record<string, unknown>,
    options?: {
      gas?: string | BN;
      attachedDeposit?: string | BN;
    }
  ) {
    const txResult = await account.call_raw(contractId, methodName, args, options);
    const successValue = txResult.parseResult();
    const outcome = txResult.result.transaction_outcome.outcome;
    const tokensBurnt = NEAR.from(outcome.gas_burnt + '000000000');
    return {
      successValue,
      metrics: {
        tokensBurnt
      }
    }
}

// This is needed due to some unknown issues of balance accuracy in sandbox
export async function numbersEqual(test: any, a: NEAR, b: NEAR, diff = 0.000001) {
  test.is(
    a.sub(b).abs().lt(NEAR.parse(diff.toString())),
    true,
    `The actual value ${a.toString()} doesn't match with expected value ${b.toString()}`
  )
}

// Match considering precision loss
export async function noMoreThanOneYoctoDiff(test: any, a: NEAR, b: NEAR, loss = "1") {
  test.is(
    a.sub(b).abs().lte(NEAR.from(loss)),
    true,
    `The actual value ${a.toString()} doesn't match with expected value ${b.toString()}`
  )
}

// Match with one of the expected values
export function matchMultipleValues(test: any, actual: any, expected: Array<any>) {
  test.true(
    expected.includes(actual),
    `The actual value ${actual} doesn't match with any of the expected values: [ ${expected.join(', ')} ]`
  );
}

export function skip(...args: any[]) {
  console.debug(`Skipping test ${args[0]} ...`);
}

export async function registerFungibleTokenUser(
  ft: NearAccount,
  user: NearAccount,
  storage_cost?: NEAR,
) {
  const storage_balance = await ft.view(
    'storage_balance_bounds',
    {}
  ) as any;
  await user.call(
    ft,
    'storage_deposit',
    { account_id: user },
    { attachedDeposit: storage_cost?.toString() || storage_balance.min.toString() },
  );
}

export function parseNEAR(a: number): NEAR {
  const yoctoString = a.toLocaleString('fullwide', { useGrouping: false });
  return NEAR.from(yoctoString);
}

export async function epochHeightFastforward(contract, user, numEpoches = NUM_EPOCHS_TO_UNLOCK) {
  // read current epoch
  let epoch: number = await contract.view('read_epoch_height', {});
  // increase epoch height
  epoch += numEpoches;
  await user.call(
    contract,
    'set_epoch_height',
    { epoch }
  );
}

export async function deployDex (root: NearAccount) {
  const contract = await root.createAndDeploy(
    'dex',
    'compiled-contracts/mock_dex.wasm',
  );
  return contract;
}

export async function setManager(root: NearAccount, contract: NearAccount, owner: NearAccount, manager?: NearAccount) {
  if (!manager) {
    manager = await root.createAccount('linear_manager', { initialBalance: NEAR.parse("1000000").toString() });
  }

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

export async function getSummary (contract: NearAccount) {
  return await contract.view("get_summary", {}) as any;
}

export async function updateBaseStakeAmounts(
  contract: NearAccount,
  manager: NearAccount,
  validator_ids: string[],
  amounts: NEAR[]
) {
  await manager.call(
    contract,
    'update_base_stake_amounts',
    {
      validator_ids,
      amounts
    }
  );
}

interface Validator {
  staked_amount: string,
  unstaked_amount: string,
  base_stake_amount: string,
  target_stake_amount: string,
  draining: boolean
}

export function getValidator(
  contract: NearAccount,
  validatorId: string
): Promise<Validator> {
  return contract.view(
    'get_validator',
    {
      validator_id: validatorId
    }
  );
}

export function assertValidatorAmountHelper (
  test: any,
  contract: NearAccount,
  owner: NearAccount
) {
  return async function (
    validator: NearAccount,
    stakedAmount: string,
    unstakedAmount: string,
    baseStakeAmount?: string,
    targetStakeAmount?: string,
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
    const v = await getValidator(contract, validator.accountId);
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

    if (baseStakeAmount) {
      const baseStaked = NEAR.from(v.base_stake_amount);
      test.is(
        baseStaked.toString(),
        NEAR.parse(baseStakeAmount).toString()
      );
    }

    if (targetStakeAmount) {
      const target = NEAR.from(v.target_stake_amount);
      test.is(
        target.toString(),
        NEAR.parse(targetStakeAmount).toString()
      );
    }
  }
}
