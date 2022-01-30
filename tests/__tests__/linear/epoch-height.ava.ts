import { initWorkSpace } from "./helper";

const workspace = initWorkSpace();

workspace.test('read/write test epoch height', async (test, {contract, alice}) => {
  test.is(
    await contract.view('read_epoch_height'),
    0,
    'init epoch height should be 0'
  );

  await alice.call(
    contract,
    'set_epoch_height',
    {
      epoch: 4
    }
  );

  test.is(
    await contract.view('read_epoch_height'),
    4,
    'epoch should be set'
  );
});
