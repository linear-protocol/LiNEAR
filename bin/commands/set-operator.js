const { init } = require("../near");

exports.command = 'set-operator <address>';
exports.desc = 'Set operator';
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
      describe: 'new operator ID'
    })
    .demandOption(['signer', 'operator'])
}

exports.handler = async function (argv) {
  const { address, operator } = argv;
  
  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  console.log(`Setting owner to ${operator}`);

  await signer.functionCall({
    contractId: address,
    methodName: 'set_operator',
    args: {
      new_operator_id: operator
    }
  });

  console.log('done');
}
