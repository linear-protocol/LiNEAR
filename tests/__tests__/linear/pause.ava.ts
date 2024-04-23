import { Gas, NEAR } from 'near-units';
import { NearAccount } from 'near-workspaces';
import {
  assertFailure,
  initWorkspace,
  ONE_YOCTO,
  registerFungibleTokenUser,
  test,
} from './helper';

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

test.beforeEach(async (t) => {
  t.context = await initWorkspace();
});

test.afterEach(async (t) => {
  await t.context.worker.tearDown();
});

test('pause/resume could not be called by non-owner', async (t) => {
  const { contract, alice } = t.context;
  await assertFailure(
    t,
    alice.call(contract, 'pause', {}),
    'Only owner can perform this action',
  );

  await assertFailure(
    t,
    alice.call(contract, 'resume', {}),
    'Only owner can perform this action',
  );
});

test('could not perform any actions when paused', async (t) => {
  const { contract, owner, alice } = t.context;
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
    t,
    transfer(contract, alice, owner, amount),
    'The contract is paused now. Please try later',
  );

  await assertFailure(
    t,
    transferCall(contract, alice, owner, amount, ''),
    'The contract is paused now. Please try later',
  );

  // cannot stake/unstake

  await assertFailure(
    t,
    alice.call(contract, 'deposit_and_stake', {}, { attachedDeposit: amount }),
    'The contract is paused now. Please try later',
  );

  await assertFailure(
    t,
    alice.call(contract, 'unstake_all', {}),
    'The contract is paused now. Please try later',
  );
});

test('resume contract after pause', async (t) => {
  const { contract, owner, alice, bob } = t.context;
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
    t,
    transfer(contract, alice, owner, transferAmount),
    'The contract is paused now. Please try later',
  );

  await owner.call(contract, 'resume', {});

  await registerFungibleTokenUser(contract, bob);
  await transfer(contract, alice, bob, transferAmount);
});
