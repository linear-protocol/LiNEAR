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
    .option('n', {
      describe: 'fraction numerator'
    })
    .option('d', {
      describe: 'fraction denominator',
      default: 10000
    })
    .demandOption(['signer', 'account', 'n'])
}

exports.handler = async function (argv) {
  const { address, n, d, account } = argv;
  
  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  console.log(`setting ${account} with fraction ${n}/${d}`);

  await signer.functionCall({
    contractId: address,
    methodName: 'set_beneficiary',
    args: {
      account_id: account,
      fraction: {
        numerator: n,
        denominator: d
      }
    }
  });

  console.log('done');
}
