const { init } = require("../near");

exports.command = 'del-bene <address>';
exports.desc = 'Remove beneficiary';
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
    .demandOption(['signer', 'account'])
}

exports.handler = async function (argv) {
  const { address, account } = argv;
  
  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  console.log(`removing beneficiary ${account}`);

  await signer.functionCall({
    contractId: address,
    methodName: 'remove_beneficiary',
    args: {
      account_id: account,
    }
  });

  console.log('done');
}
