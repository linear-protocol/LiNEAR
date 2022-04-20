const { init } = require("../near");

exports.command = 'remove-node <address>';
exports.desc = 'Remove node';
exports.builder = yargs => {
  yargs
    .positional('address', {
      describe: 'Contract address',
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
    .option('node', {
      describe: 'node ID to remove'
    })
    .demandOption(['signer', 'node'])
}

exports.handler = async function (argv) {
  const { address, node } = argv;

  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  console.log('Removing node ', node);

  await signer.functionCall({
    contractId: address,
    methodName: 'remove_validator',
    args: {
      validator_id: node
    }
  });

  console.log('done');
}
