import { Workspace, NearAccount, } from "near-workspaces-ava";

interface RewardFee {
  numerator: number,
  denominator: number
}

export function initWorkSpace() {
  return Workspace.init(async ({ root }) => {
    const owner = await root.createAccount('linear_owner');
    const alice = await root.createAccount('alice');
    const bob = await root.createAccount('bob');

    const contract = await deployLinear(root, owner.accountId);

    return { contract, owner, alice, bob };
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
      let msg: string = e.kind.ExecutionError;
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

export function skip(...args: any[]) {};

export async function registerFungibleTokenUser(ft: NearAccount, user: NearAccount) {
  const storage_balance = await ft.view(
    'storage_balance_bounds',
    {}
  ) as any;
  await user.call(
    ft,
    'storage_deposit',
    { account_id: user },
    { attachedDeposit: storage_balance.min.toString() },
  );
}
