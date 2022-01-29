import { Workspace, NearAccount } from "near-workspaces-ava";

interface RewardFee {
  numerator: number,
  denominator: number
}

export function initWorkSpace() {
  return Workspace.init(async ({ root }) => {
    const owner = await root.createAccount('linear_owner');
    const alice = await root.createAccount('alice');

    const contract = await deployLinear(root, owner.accountId);

    return { contract, alice };
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
