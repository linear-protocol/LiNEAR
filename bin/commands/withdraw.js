const { Gas } = require("near-units");
const { init } = require("../near");
const { networkOption } = require("./common");

exports.command = 'withdraw <address>';
exports.desc = 'Withdraw from a validator, for testing purpose';
exports.builder = yargs => {
  yargs
    .positional('address', {
      describe: 'Contract address to deploy to',
      type: 'string'
    })
    .option('network', networkOption)
    .demandOption(['signer', 'validator'])
    .option('signer', {
      describe: 'signer account Id to call contract'
    })
    .option('validator', {
      describe: 'validator ID'
    })
}

exports.handler = async function (argv) {
  const { address, validator } = argv;

  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  const outcome = await signer.functionCall({
    contractId: address,
    methodName: 'epoch_withdraw',
    args: {
      validator_id: validator
    },
    gas: Gas.parse('200 Tgas')
  });

  console.log(outcome);
  console.log('withdrawn');
}
