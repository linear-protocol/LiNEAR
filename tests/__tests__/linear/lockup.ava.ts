import { NearAccount, NEAR, Gas } from 'near-workspaces';
import {createAndDeploy, initWorkSpace, test} from './helper';

const workspace = initWorkSpace();

async function createLockupAccount(root: NearAccount, owner: NearAccount) {
  const lockupAccount = await createAndDeploy(
    root,
    'lockup',
    'compiled-contracts/mock_lockup.wasm',
    {
      balance: '10000 N'
    }
  )

  const startTs = Date.now() + 360000; // starts some time in the future
  const cliffTs = startTs + 360000;
  const endTs = cliffTs + 360000;

  // init lockup
  await owner.call(lockupAccount, 'new', {
    owner_account_id: owner.accountId,
    staking_pool_whitelist_account_id: 'whitelist',
    lockup_duration: '0',
    transfers_information: {
      TransfersEnabled: {
        transfers_timestamp: '1602614338293769340',
      },
    },
    vesting_schedule: {
      VestingSchedule: {
        start_timestamp: `${startTs}000000`,
        cliff_timestamp: `${cliffTs}000000`,
        end_timestamp: `${endTs}000000`,
      },
    },
    foundation_account_id: owner.accountId,
  });

  return lockupAccount;
}

test.before(async (t) => {
  t.context = await initWorkSpace();
});

test.after(async (t) => {
  await t.context.worker.tearDown();
});

test(
  'Lockup account stake',
  async (t) => {
    const { root, contract, alice, owner } = t.context;
    const lockupAccount = await createLockupAccount(root, alice);
    const lockedAmount: string = await lockupAccount.view('get_locked_amount');

    // the owner balance should be very small now
    t.true(
      NEAR.from(await lockupAccount.view('get_owners_balance'))
        .sub(NEAR.parse('0.01'))
        .toBigInt() < 0,
      'initial owner balance too large',
    );

    await alice.call(lockupAccount, 'select_staking_pool', {
      staking_pool_account_id: contract.accountId,
    });

    const poolId = await lockupAccount.view('get_staking_pool_account_id');
    t.is(
      poolId,
      contract.accountId,
      'staking pool id should be contract id',
    );

    const storageReserve = NEAR.parse('3.5');
    const amountToStake = NEAR.from(lockedAmount)
      .sub(storageReserve)
      .toString(10);

    await alice.call(
      lockupAccount,
      'deposit_and_stake',
      {
        amount: amountToStake,
      },
      {
        gas: Gas.parse('150 Tgas'),
      },
    );

    // check staked NEAR amount
    t.is(
      await contract.view('get_account_staked_balance', {
        account_id: lockupAccount.accountId,
      }),
      amountToStake,
    );
    // check minted LiNEAR amount
    t.is(
      await contract.view('ft_balance_of', {
        account_id: lockupAccount.accountId,
      }),
      amountToStake,
    );

    // generate rewards
    const rewards = NEAR.parse('100');
    await owner.call(contract, 'add_epoch_rewards', {
      amount: rewards.toString(10),
    });

    await alice.call(
      lockupAccount,
      'refresh_staking_pool_balance',
      {},
      {
        gas: Gas.parse('150 Tgas'),
      },
    );

    const lockAccountNewBalance: string = await lockupAccount.view(
      'get_known_deposited_balance',
    );
    const newStakedBalance: string = await contract.view(
      'get_account_staked_balance',
      {
        account_id: lockupAccount.accountId,
      },
    );
    t.is(
      lockAccountNewBalance,
      newStakedBalance,
      'Lockup account should refresh balance from staking pool',
    );

    // unstake
    const amountToUnstake = NEAR.parse('1000');
    await alice.call(
      lockupAccount,
      'unstake',
      {
        amount: amountToUnstake.toString(10),
      },
      {
        gas: Gas.parse('150 Tgas'),
      },
    );

    // almost all rewards should be the owner's available balance now
    t.true(
      NEAR.from(await lockupAccount.view('get_owners_balance'))
        .sub(NEAR.parse('99'))
        .toBigInt() > 0,
    );

    // fast-forward
    await owner.call(contract, 'set_epoch_height', { epoch: 14 });

    // withdraw
    await alice.call(
      lockupAccount,
      'withdraw_all_from_staking_pool',
      {},
      {
        gas: Gas.parse('175 Tgas'),
      },
    );
  },
);
