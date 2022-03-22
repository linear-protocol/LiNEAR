const { init } = require("../near");

exports.command = 'del-operator <address>';
exports.desc = 'Remove operator';
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
    .option('operator', {
      describe: 'operator ID to remove'
    })
    .demandOption(['signer', 'operator'])
}

exports.handler = async function (argv) {
  const { address, operator } = argv;
  
  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  console.log(`Removing operator ${operator}`);

  await signer.functionCall({
    contractId: address,
    methodName: 'remove_operator',
    args: {
      operator_id: operator
    }
  });

  console.log('done');
}
