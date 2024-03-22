import {initWorkSpace, test} from './helper';

test.beforeEach(async (t) => {
  t.context = await initWorkSpace();
});

test.afterEach(async (t) => {
  await t.context.worker.tearDown();
});

test(
  'read/write test epoch height',
  async (t) => {
    const  { contract, alice } = t.context;
    t.is(
      await contract.view('read_epoch_height'),
      10,
      'init epoch height should be 10',
    );

    await alice.call(contract, 'set_epoch_height', {
      epoch: 14,
    });

    t.is(
      await contract.view('read_epoch_height'),
      14,
      'epoch should be set',
    );
  },
);
