const { init } = require("../near");

exports.command = 'set-tsry <address>';
exports.desc = 'Set treasury';
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
    .option('account', {
      describe: 'new treasury account ID'
    })
    .demandOption(['signer', 'account'])
}

exports.handler = async function (argv) {
  const { address, account } = argv;
  
  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  console.log(`Setting owner to ${owner}`);

  await signer.functionCall({
    contractId: address,
    methodName: 'set_treasury',
    args: {
      account_id: account
    }
  });

  console.log('done');
}
