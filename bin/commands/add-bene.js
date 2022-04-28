const { init } = require("../near");

exports.command = 'add-bene <address>';
exports.desc = 'Add beneficiary';
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
      describe: 'beneficiary account ID'
    })
    .option('percent', {
      describe: 'percentage based on 10000'
    })
    .demandOption(['signer', 'account', 'percent'])
}

exports.handler = async function (argv) {
  const { address, percent, account } = argv;
  
  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  console.log(`setting ${account} with fraction ${n}/${d}`);

  await signer.functionCall({
    contractId: address,
    methodName: 'set_beneficiary',
    args: {
      account_id: account,
      percent: parseInt(percent)
    }
  });

  console.log('done');
}
