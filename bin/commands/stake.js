const { init } = require("../near");
const { NEAR }  = require('near-units');
const { networkOption } = require("./common");

exports.command = 'stake <address>';
exports.desc = 'Deposit and Stake, for testing purpose';
exports.builder = yargs => {
  yargs
    .positional('address', {
      describe: 'Contract address to deploy to',
      type: 'string'
    })
    .option('network', networkOption)
    .demandOption(['signer'])
    .option('signer', {
      describe: 'signer account Id to call contract'
    })
    .option('amount', {
      describe: 'Deposit amount in NEAR',
      default: '10'
    })
}

exports.handler = async function (argv) {
  const address = argv.address;

  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  await signer.functionCall({
    contractId: address,
    methodName: 'deposit_and_stake',
    args: {},
    attachedDeposit: NEAR.parse(argv.amount)
  });

  console.log('staked');
}
