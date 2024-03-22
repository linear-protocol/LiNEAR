import {initWorkSpace, test} from './helper';

interface ContractSourceMetadata {
  version: String;
  link: String;
}

test.before(async (t) => {
  t.context = await initWorkSpace();
});

test.after(async (t) => {
  await t.context.worker.tearDown();
});

test('read contract source metadata', async (t) => {
  const { contract } = t.context;
  t.is(
    (
      (await contract.view(
        'contract_source_metadata',
        {},
      )) as ContractSourceMetadata
    ).link,
    'https://github.com/linear-protocol/LiNEAR',
  );
});
