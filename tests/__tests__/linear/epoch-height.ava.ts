import { initWorkSpace } from "./helper";

const workspace = initWorkSpace();

workspace.test('read/write test epoch height', async (test, {contract, alice}) => {
  test.is(
    await contract.view('read_epoch_height'),
    10,
    'init epoch height should be 10'
  );

  await alice.call(
    contract,
    'set_epoch_height',
    {
      epoch: 14
    }
  );

  test.is(
    await contract.view('read_epoch_height'),
    14,
    'epoch should be set'
  );
});
