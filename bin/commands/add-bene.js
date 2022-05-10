const { init, funcCall } = require("../near");

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
    .option('bps', {
      describe: 'basis point of 10000'
    })
    .demandOption(['signer', 'account', 'bps'])
    .option('dao', {
      describe: 'DAO address'
    })
}

exports.handler = async function (argv) {
  const { address, bps, account, dao } = argv;
  
  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  console.log(`setting ${account} with bps ${bps}`);

  const args = {
    account_id: account,
    bps: parseInt(bps)
  };

  await funcCall(
    signer,
    dao,
    `Add beneficiary ${account}`,
    address,
    'set_beneficiary',
    args,
  );

  console.log('done');
}
