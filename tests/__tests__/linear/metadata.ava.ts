import { initWorkspace, test } from './helper';

interface ContractSourceMetadata {
  version: String;
  link: String;
}

test.beforeEach(async (t) => {
  t.context = await initWorkspace();
});

test.afterEach(async (t) => {
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
