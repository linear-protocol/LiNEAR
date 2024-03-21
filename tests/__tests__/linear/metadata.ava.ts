import { initWorkSpace } from './helper';

const workspace = initWorkSpace();

interface ContractSourceMetadata {
  version: String;
  link: String;
}

workspace.test('read contract source metadata', async (test, { contract }) => {
  test.is(
    (
      (await contract.view(
        'contract_source_metadata',
        {},
      )) as ContractSourceMetadata
    ).link,
    'https://github.com/linear-protocol/LiNEAR',
  );
});
