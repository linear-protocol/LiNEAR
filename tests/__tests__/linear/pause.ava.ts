import { Gas, NEAR } from 'near-units';
import { NearAccount } from 'near-workspaces-ava';
import {
  assertFailure,
  initWorkSpace,
  ONE_YOCTO,
  registerFungibleTokenUser,
} from './helper';

const workspace = initWorkSpace();

async function transfer(
  contract: NearAccount,
  sender: NearAccount,
  receiver: NearAccount,
  amount: NEAR,
) {
  await sender.call(
    contract,
    'ft_transfer',
    {
      receiver_id: receiver,
      amount,
    },
    {
      attachedDeposit: ONE_YOCTO,
    },
  );
}

async function transferCall(
  contract: NearAccount,
  sender: NearAccount,
  receiver: NearAccount,
  amount: NEAR,
  msg: String,
  memo?: String,
) {
  await sender.call(
    contract,
    'ft_transfer_call',
    {
      receiver_id: receiver,
      amount,
      memo,
      msg,
    },
    {
      gas: Gas.parse('75 Tgas'),
      attachedDeposit: ONE_YOCTO,
    },
  );
}

workspace.test(
  'pause/resume could not be called by non-owner',
  async (test, { contract, alice }) => {
    await assertFailure(
      test,
      alice.call(contract, 'pause', {}),
      'Only owner can perform this action',
    );

    await assertFailure(
      test,
      alice.call(contract, 'resume', {}),
      'Only owner can perform this action',
    );
  },
);

workspace.test(
  'could not perform any actions when paused',
  async (test, { contract, owner, alice }) => {
    // deposit and stake 10 NEAR
    const stakeAmount = NEAR.parse('10');
    await alice.call(
      contract,
      'deposit_and_stake',
      {},
      { attachedDeposit: stakeAmount },
    );

    await owner.call(contract, 'pause', {});

    // cannot transfer LiNEAR

    const amount = NEAR.parse('2');
    await assertFailure(
      test,
      transfer(contract, alice, owner, amount),
      'The contract is paused now. Please try later',
    );

    await assertFailure(
      test,
      transferCall(contract, alice, owner, amount, ''),
      'The contract is paused now. Please try later',
    );

    // cannot stake/unstake

    await assertFailure(
      test,
      alice.call(
        contract,
        'deposit_and_stake',
        {},
        { attachedDeposit: amount },
      ),
      'The contract is paused now. Please try later',
    );

    await assertFailure(
      test,
      alice.call(contract, 'unstake_all', {}),
      'The contract is paused now. Please try later',
    );
  },
);

workspace.test(
  'resume contract after pause',
  async (test, { contract, owner, alice, bob }) => {
    // deposit and stake 10 NEAR
    const stakeAmount = NEAR.parse('10');
    await alice.call(
      contract,
      'deposit_and_stake',
      {},
      { attachedDeposit: stakeAmount },
    );

    await owner.call(contract, 'pause', {});

    const transferAmount = NEAR.parse('2');
    await assertFailure(
      test,
      transfer(contract, alice, owner, transferAmount),
      'The contract is paused now. Please try later',
    );

    await owner.call(contract, 'resume', {});

    await registerFungibleTokenUser(contract, bob);
    await transfer(contract, alice, bob, transferAmount);
  },
);
