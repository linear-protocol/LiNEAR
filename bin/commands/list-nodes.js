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
    .option('signer', {
      describe: 'signer account Id to call contract'
    })
    .demandOption(['signer'])
}

exports.handler = async function (argv) {
  const address = argv.address;

  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  // currentNodes is a map from nodeID to validator struct
  const currentNodes = await getValidators(signer, address);
  console.log(currentNodes);
}

async function getValidators(signer, address) {
  let results = {};
  let offset = 0;
  const limit = 20;

  while (true) {
    const data = await signer.functionCall({
      contractId: address,
      methodName: 'get_validators',
      args: {
        offset,
        limit
      }
    });

    const rawValue = data.status.SuccessValue;
    const rawString = Buffer.from(rawValue, 'base64').toString();
    const res = JSON.parse(rawString);
    if (res.length === 0) break;

    offset += res.length;

    for (const node of res) {
      results[node.account_id] = node;
    }
  }

  return results;
}
