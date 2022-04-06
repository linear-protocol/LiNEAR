const { init } = require("../near");
const { Gas } = require("near-units");

exports.command = 'drain-unstake <address>';
exports.desc = 'Drain Unstake';
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
    .option('validator', {
      describe: 'Validator ID to unstake'
    })
    .demandOption(['signer', 'validator'])
}

exports.handler = async function (argv) {
  const {
    address,
    network,
  } = argv;

  const near = await init(network);
  const signer = await near.account(argv.signer);

  console.log(`Drain unstake from ${argv.validator}`);

  await signer.functionCall({
    contractId: address,
    methodName: 'drain_unstake',
    args: {
      validator_id: argv.validator
    },
    gas: Gas.parse('200 Tgas')
  });

  console.log('done');
}
