const { init, funcCall } = require("../near");
const prompts = require('prompts');

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
    .option('dao', {
      describe: 'DAO address'
    })
}

exports.handler = async function (argv) {
  const { address, account, dao } = argv;
  
  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  console.log(`Setting treasury to ${account}`);

  const res = await prompts({
    type: 'confirm',
    name: 'confirm',
    message: 'Confirm update?'
  });
  if (!res.confirm) return;

  await funcCall(
    signer,
    dao,
    `Set treasury to ${account}`,
    address,
    'set_treasury',
    {
      account_id: account
    }
  );

  console.log('done');
}
