const { init, funcCall } = require("../near");
const prompts = require('prompts');
const { networkOption } = require("./common");

exports.command = 'set-whitelist <address>';
exports.desc = 'Set whitelist';
exports.builder = yargs => {
  yargs
    .positional('address', {
      describe: 'Contract address to deploy to',
      type: 'string'
    })
    .option('network', networkOption)
    .option('signer', {
      describe: 'signer account Id to call contract'
    })
    .option('whitelist', {
      describe: 'whitelist contract ID'
    })
    .demandOption(['signer', 'whitelist'])
    .option('dao', {
      describe: 'DAO address'
    })
}

exports.handler = async function (argv) {
  const { address, whitelist, dao } = argv;
  
  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  console.log(`Setting whitelist to ${whitelist}`);

  const res = await prompts({
    type: 'confirm',
    name: 'confirm',
    message: 'Confirm update?'
  });
  if (!res.confirm) return;

  await funcCall(
    signer,
    dao,
    `Set whitelist to ${whitelist}`,
    address,
    'set_whitelist_contract_id',
    {
      account_id: whitelist
    }
  );


  console.log('done');
}
