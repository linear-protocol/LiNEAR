const { init } = require('../near');

exports.command = 'list-nodes <address>';
exports.desc = 'List validators of the contract';
exports.builder = yargs => {
  yargs
    .positional('address', {
      describe: 'Contract address to deploy to',
      type: 'string'
    })
    .option('network', {
      describe: 'network ID',
      default: 'testnet',
      choices: ['testnet', 'mainnet']
    })
}

exports.handler = async function (argv) {
  const address = argv.address;

  const near = await init(argv.network);
  const contract = await near.account(address);

  // currentNodes is a map from nodeID to validator struct
  const currentNodes = await getValidators(contract);
  console.log(currentNodes);
}

async function getValidators(contract) {
  let results = {};
  let offset = 0;
  const limit = 20;

  while (true) {
    const res = await contract.viewFunction(
      contract.accountId,
      'get_validators',
      {
        offset,
        limit
      }
    );
    if (res.length === 0) break;

    offset += res.length;

    for (const node of res) {
      results[node.account_id] = node;
    }
  }

  return results;
}
